use std::fmt::Display;
use std::fs;
use std::intrinsics::transmute;
use std::iter::{once, repeat};
use std::ops::Deref;
use std::sync::Arc;

use ahash::AHashMap;
use anyhow::{bail, Result};
use basedb::diagnostics::{
    Config, ConsoleSink, Diagnostic, DiagnosticSink, Label, LabelStyle, Report, Severity,
};
use basedb::lints::{Lint, LintLevel};
use basedb::{BaseDB, BaseDatabase, FileId, Upcast, Vfs, VfsPath, VfsStorage, STANDARD_FLAGS};
use hir_def::db::{HirDefDB, HirDefDatabase, InternDatabase};
use hir_def::nameres::ScopeDefItem;
use hir_def::{Lookup, ModuleId, ParamId, Path, ScopeId, Type, VarId};
use hir_ty::db::HirTyDatabase;
use hir_ty::{collect_diagnostics, collect_path, visit_relative_defs};
use indexmap::IndexMap;
use parking_lot::RwLock;
use salsa::ParallelDatabase;
use smol_str::SmolStr;
use stdx::iter::zip;
use syntax::ast::{AttrsOwner, Expr, LiteralKind, ParamDecl, PathExpr, VarDecl};
use syntax::sourcemap::FileSpan;
use syntax::{AstNode, SyntaxNode, TextRange};
use typed_index_collections::TiSlice;

use crate::opts::abs_path;
use crate::Opts;

#[salsa::database(BaseDatabase, InternDatabase, HirDefDatabase, HirTyDatabase)]
pub(crate) struct CompilationDB {
    storage: salsa::Storage<CompilationDB>,
    pub vfs: Arc<RwLock<Vfs>>,
    pub root_file: FileId,
}

impl Upcast<dyn HirDefDB> for CompilationDB {
    fn upcast(&self) -> &(dyn HirDefDB + 'static) {
        &*self
    }
}

impl Upcast<dyn BaseDB> for CompilationDB {
    fn upcast(&self) -> &(dyn BaseDB + 'static) {
        self
    }
}

impl CompilationDB {
    // TODO configure
    pub(crate) fn new(root_file: &std::path::Path, opts: &Opts) -> Result<Self> {
        let mut vfs = Vfs::default();
        vfs.insert_std_lib();

        let root_file = if let Some(vfs_export) = opts.vfs()? {
            for (path, data) in vfs_export {
                vfs.add_virt_file(path, data.to_owned().into());
            }
            let root_file = root_file.to_str();
            let root_file = match root_file {
                Some(file) => file,
                None => bail!("For VFS operations all paths must be representable as utf8!"),
            };

            if !root_file.starts_with('/') {
                bail!("VFS paths must start with '/'")
            }

            let root_file_id = vfs.file_id(&VfsPath::new_virtual_path(root_file.to_owned()));
            match root_file_id {
                Some(id) => id,
                None => bail!("paht '{}' is not present in the VFS!", root_file),
            }
        } else {
            let root_file = abs_path(root_file)?;
            let contents = fs::read(&root_file);
            let root_file = vfs.ensure_file_id(root_file.into());
            vfs.set_file_contents(root_file, contents.into());
            root_file
        };

        let mut res =
            Self { storage: salsa::Storage::default(), vfs: Arc::new(RwLock::new(vfs)), root_file };

        let include_dirs: Result<Arc<[_]>> = once(Ok(VfsPath::new_virtual_path("/std".to_owned())))
            .chain(opts.include_dirs().map(|it| Ok(VfsPath::from(it?))))
            .collect();
        res.set_include_dirs(root_file, include_dirs?);

        let macro_flags: Vec<_> =
            STANDARD_FLAGS.into_iter().chain(opts.macro_flags()).map(Arc::from).collect();
        res.set_macro_flags(root_file, Arc::from(macro_flags));

        res.set_plugin_lints(&[]);
        let mut overwrites = res.empty_global_lint_overwrites();
        let registry = res.lint_registry();

        let allow_lints = zip(opts.allow_lints(), repeat(LintLevel::Allow));
        let warn_lints = zip(opts.warn_lints(), repeat(LintLevel::Warn));
        let deny_lints = zip(opts.deny_lints(), repeat(LintLevel::Deny));

        let mut sink = ConsoleSink::new(Config::default(), &res);
        for (lint, lvl) in allow_lints.chain(warn_lints).chain(deny_lints) {
            if let Some(lint) = registry.lint_from_name(lint) {
                overwrites[lint] = Some(lvl)
            } else {
                sink.print_simple_message(
                    Severity::Warning,
                    format!("no lint named '{}' was found!", lint),
                )
            }
        }
        drop(sink);

        let overwrites: Arc<[_]> = Arc::from(overwrites.as_ref());
        let overwrites = unsafe {
            transmute::<Arc<[Option<LintLevel>]>, Arc<TiSlice<Lint, Option<LintLevel>>>>(overwrites)
        };

        res.set_global_lint_overwrites(root_file, overwrites);
        Ok(res)
    }
}

impl ParallelDatabase for CompilationDB {
    fn snapshot(&self) -> salsa::Snapshot<Self> {
        let db = CompilationDB {
            storage: self.storage.snapshot(),
            vfs: self.vfs.clone(),
            root_file: self.root_file,
        };

        salsa::Snapshot::new(db)
    }
}

/// This impl tells salsa where to find the salsa runtime.
impl salsa::Database for CompilationDB {}
impl VfsStorage for CompilationDB {
    fn vfs(&self) -> &RwLock<Vfs> {
        &self.vfs
    }
}

struct IllegalExpr {
    expr: Expr,
    expected: &'static str,
    attr: &'static str,
}

impl Diagnostic for IllegalExpr {
    fn build_report(&self, root_file: FileId, db: &dyn BaseDB) -> Report {
        let FileSpan { range, file } = db
            .parse(root_file)
            .to_file_span(self.expr.syntax().text_range(), &db.sourcemap(root_file));
        Report::error()
            .with_message(format!(
                "illegal expression supplied to '{}' attribute; expected {}",
                self.attr, self.expected
            ))
            .with_labels(vec![Label {
                style: LabelStyle::Primary,
                file_id: file,
                range: range.into(),
                message: "illegal expression".to_owned(),
            }])
    }
}

struct IllegalType {
    range: TextRange,
    allowed: &'static str,
}

impl Diagnostic for IllegalType {
    fn build_report(&self, root_file: FileId, db: &dyn BaseDB) -> Report {
        let FileSpan { range, file } =
            db.parse(root_file).to_file_span(self.range, &db.sourcemap(root_file));
        Report::error()
            .with_message(format!("VerilogAE only supports {}", self.allowed))
            .with_labels(vec![Label {
                style: LabelStyle::Primary,
                file_id: file,
                range: range.into(),
                message: "unsupported type".to_owned(),
            }])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub var: VarId,
    pub dependency_breaking: Box<[VarId]>,
    pub prefix: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamInfo {
    pub name: SmolStr,
    pub unit: String,
    pub description: String,
    pub group: String,
    pub ty: Type,
}

pub struct ModelInfo {
    pub params: IndexMap<ParamId, ParamInfo>,
    pub functions: Vec<Function>,
    pub var_names: AHashMap<VarId, SmolStr>,
    pub op_vars: Vec<SmolStr>,
    pub module: ModuleId,
}

impl ModelInfo {
    pub(crate) fn collect(
        db: &CompilationDB,
        file_name: &impl Display,
        name: Option<&str>,
    ) -> Result<Self> {
        let root_file = db.root_file;

        let mut sink = ConsoleSink::new(Config::default(), db.upcast());
        sink.add_diagnostics(&*db.preprocess(root_file).diagnostics, root_file, db);
        sink.add_diagnostics(db.parse(root_file).errors(), root_file, db);
        collect_diagnostics(db, root_file, &mut sink);

        if sink.summary(file_name) {
            bail!("compiation failed");
        }

        let root_def_map = db.def_map(root_file);
        let module = root_def_map[root_def_map.entry()]
            .declarations
            .iter()
            .find_map(|(def_name, def)| {
                if let ScopeDefItem::ModuleId(module) = def {
                    if let Some(name) = name {
                        if name != def_name.deref() {
                            return None;
                        }
                    }
                    Some(module)
                } else {
                    None
                }
            })
            .copied();

        let module = match module {
            Some(module) => module,
            None => {
                let msg = if let Some(name) = name {
                    format!("failed to find module {}", name)
                } else {
                    "no module was found".to_owned()
                };

                bail!("{}", msg)
            }
        };

        let mut params = IndexMap::default();
        let mut functions = Vec::new();
        let mut var_names = AHashMap::new();
        let mut op_vars = Vec::new();

        let parse = db.parse(root_file);
        let ast_id_map = db.ast_id_map(root_file);
        let sm = db.sourcemap(root_file);

        let mut resolved_retrieve_attr = AHashMap::new();
        let mut resolved_param_attrs = AHashMap::new();

        let resolve_path = |sink: &mut ConsoleSink, expr: PathExpr, scope: ScopeId| {
            let path = Path::resolve(expr.path().unwrap()).unwrap();
            match scope.resolve_item_path(db.upcast(), &path) {
                Ok(var) => Some(var),
                Err(err) => {
                    let FileSpan { range, file } =
                        parse.to_file_span(expr.syntax().text_range(), &sm);
                    let report =
                        Report::error().with_message(err.to_string()).with_labels(vec![Label {
                            style: LabelStyle::Primary,
                            file_id: file,
                            range: range.into(),
                            message: err.message(),
                        }]);

                    sink.add_report(report);
                    None
                }
            }
        };

        let resolve_str_attr = |sink: &mut ConsoleSink, ast: &ParamDecl, attr_name| {
            let val = ast.get_attr(attr_name)?.val()?;
            if let Expr::Literal(lit) = &val {
                if let LiteralKind::String(lit) = lit.kind() {
                    return Some(lit.unescaped_value());
                }
            }

            let diag = IllegalExpr { expr: val, expected: "a string literal", attr: attr_name };
            sink.add_diagnostic(&diag, root_file, db.upcast());
            None
        };

        let mut counter = 0;

        let check_numeric = |allowed, var, syntax: &SyntaxNode, sink: &mut ConsoleSink| {
            if !matches!(db.var_data(var).ty, Type::Real | Type::Integer) {
                sink.add_diagnostic(
                    &IllegalType { range: syntax.text_range(), allowed },
                    root_file,
                    db,
                );
            }
        };

        visit_relative_defs(db, module.lookup(db.upcast()).scope, |path, name, def| match def {
            ScopeDefItem::VarId(var) => {
                let loc = var.lookup(db.upcast());
                let ast = loc.item_tree(db.upcast())[loc.id].ast_id;
                let ast = ast_id_map.get(ast).to_node(parse.tree().syntax());
                let ast = VarDecl::cast(ast.syntax().parent().unwrap()).unwrap();
                let name = collect_path(path, name);

                let mut resolve_retrieve = || {
                    let attr = ast.get_attr("retrieve")?;
                    let res = if let Some(val) = attr.val() {
                        check_numeric(
                            "calculating real and integer variables",
                            var,
                            ast.syntax(),
                            &mut sink,
                        );
                        match val {
                            Expr::PathExpr(expr) => {
                                if let Some(dep) = resolve_path(&mut sink, expr.clone(), loc.scope)
                                {
                                    check_numeric("breaking dependencies on real and integer variables", dep, expr.syntax(), &mut sink);
                                    vec![dep]
                                } else {
                                    vec![]
                                }
                            }
                            Expr::ArrayExpr(expr) => expr
                                .exprs()
                                .filter_map(|expr| {
                                    if let Expr::PathExpr(expr) = expr {
                                        let res = resolve_path(&mut sink, expr.clone(), loc.scope);
                                        if let Some(dep) = res{
                                            check_numeric("breaking dependencies on real and integer variables", dep, expr.syntax(), &mut sink);
                                        }
                                        res
                                    } else {
                                        let diag = IllegalExpr {
                                            expr,
                                            expected: "a path",
                                            attr: "retrieve",
                                        };
                                        sink.add_diagnostic(&diag, root_file, db.upcast());
                                        None
                                    }
                                })
                                .collect(),
                            expr => {
                                let diag = IllegalExpr {
                                    expr,
                                    expected: "a path or an array",
                                    attr: "retrieve",
                                };
                                sink.add_diagnostic(&diag, root_file, db.upcast());
                                vec![]
                            }
                        }
                    } else {
                        vec![]
                    };

                    Some(res.into_boxed_slice())
                };

                let resolved = resolved_retrieve_attr
                    .entry(ast.syntax().text_range())
                    .or_insert_with(|| (resolve_retrieve(), ast.has_attr("op_var")));

                if let Some(resolved) = &resolved.0 {
                    functions.push(Function {
                        var,
                        dependency_breaking: resolved.clone(),
                        prefix: format!(
                            "fun.{}",
                            base_n::encode(counter, base_n::ALPHANUMERIC_ONLY)
                        ),
                    });
                    counter += 1;
                }

                if resolved.1 {
                    op_vars.push(name.clone())
                }

                var_names.insert(var, name);
            }

            ScopeDefItem::ParamId(param) => {
                // TODO ParamInfo
                let loc = param.lookup(db.upcast());
                let ast = loc.item_tree(db.upcast())[loc.id].ast_id;
                let ast = ast_id_map.get(ast).to_node(parse.tree().syntax());
                let ast = ParamDecl::cast(ast.syntax().parent().unwrap()).unwrap();

                let range = ast.syntax().text_range();

                let resolve_param_info = || {
                    let ty = db.param_data(param).ty.clone();
                    if !matches!(ty, Type::Real | Type::Integer | Type::String) {
                        sink.add_diagnostic(
                            &IllegalType { range, allowed: "real, integer and string parameters" },
                            root_file,
                            db,
                        );
                    }
                    ParamInfo {
                        name: SmolStr::new_inline(""),
                        unit: resolve_str_attr(&mut sink, &ast, "units").unwrap_or_default(),
                        description: resolve_str_attr(&mut sink, &ast, "desc").unwrap_or_default(),
                        group: resolve_str_attr(&mut sink, &ast, "group").unwrap_or_default(),
                        ty,
                    }
                };

                let mut info =
                    resolved_param_attrs.entry(range).or_insert_with(resolve_param_info).clone();

                info.name = collect_path(path, name);
                params.insert(param, info);
            }
            _ => (),
        });

        if sink.summary(file_name) {
            bail!("compilation failed");
        }

        Ok(ModelInfo { params, functions, op_vars, module, var_names })
    }
}

//! Generated by `gen_osdi_structs`, do not edit by hand.

use mir_llvm::CodegenCx;

const STDLIB_BITCODE_X86_64_UNKNOWN_LINUX_GNU: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/stdlib_0_4_x86_64-unknown-linux-gnu.bc"));
const STDLIB_BITCODE_X86_64_PC_WINDOWS_MSVC: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/stdlib_0_4_x86_64-pc-windows-msvc.bc"));
const STDLIB_BITCODE_X86_64_APPLE_MACOSX10_15_0: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/stdlib_0_4_x86_64-apple-macosx10.15.0.bc"));
const STDLIB_BITCODE_AARCH64_UNKNOWN_LINUX_GNU: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/stdlib_0_4_aarch64-unknown-linux-gnu.bc"));
const STDLIB_BITCODE_AARCH64_PC_WINDOWS_MSVC: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/stdlib_0_4_aarch64-pc-windows-msvc.bc"));
const STDLIB_BITCODE_ARM64_APPLE_MACOSX11_0_0: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/stdlib_0_4_arm64-apple-macosx11.0.0.bc"));
pub fn stdlib_bitcode(target: &target::spec::Target) -> &'static [u8] {
    match &*target.llvm_target {
        "x86_64-unknown-linux-gnu" => STDLIB_BITCODE_X86_64_UNKNOWN_LINUX_GNU,
        "x86_64-pc-windows-msvc" => STDLIB_BITCODE_X86_64_PC_WINDOWS_MSVC,
        "x86_64-apple-macosx10.15.0" => STDLIB_BITCODE_X86_64_APPLE_MACOSX10_15_0,
        "aarch64-unknown-linux-gnu" => STDLIB_BITCODE_AARCH64_UNKNOWN_LINUX_GNU,
        "aarch64-pc-windows-msvc" => STDLIB_BITCODE_AARCH64_PC_WINDOWS_MSVC,
        "arm64-apple-macosx11.0.0" => STDLIB_BITCODE_ARM64_APPLE_MACOSX11_0_0,
        triple => unreachable!("unknown target triple {triple}"),
    }
}
pub const OSDI_VERSION_MAJOR_CURR: u32 = 0;
pub const OSDI_VERSION_MINOR_CURR: u32 = 3;
pub const PARA_TY_MASK: u32 = 3;
pub const PARA_TY_REAL: u32 = 0;
pub const PARA_TY_INT: u32 = 1;
pub const PARA_TY_STR: u32 = 2;
pub const PARA_KIND_MASK: u32 = (3 << 30);
pub const PARA_KIND_MODEL: u32 = (0 << 30);
pub const PARA_KIND_INST: u32 = (1 << 30);
pub const PARA_KIND_OPVAR: u32 = (2 << 30);
pub const ACCESS_FLAG_READ: u32 = 0;
pub const ACCESS_FLAG_SET: u32 = 1;
pub const ACCESS_FLAG_INSTANCE: u32 = 4;
pub const JACOBIAN_ENTRY_RESIST_CONST: u32 = 1;
pub const JACOBIAN_ENTRY_REACT_CONST: u32 = 2;
pub const JACOBIAN_ENTRY_RESIST: u32 = 4;
pub const JACOBIAN_ENTRY_REACT: u32 = 8;
pub const CALC_RESIST_RESIDUAL: u32 = 1;
pub const CALC_REACT_RESIDUAL: u32 = 2;
pub const CALC_RESIST_JACOBIAN: u32 = 4;
pub const CALC_REACT_JACOBIAN: u32 = 8;
pub const CALC_NOISE: u32 = 16;
pub const CALC_OP: u32 = 32;
pub const CALC_RESIST_LIM_RHS: u32 = 64;
pub const CALC_REACT_LIM_RHS: u32 = 128;
pub const ENABLE_LIM: u32 = 256;
pub const INIT_LIM: u32 = 512;
pub const ANALYSIS_NOISE: u32 = 1024;
pub const ANALYSIS_DC: u32 = 2048;
pub const ANALYSIS_AC: u32 = 4096;
pub const ANALYSIS_TRAN: u32 = 8192;
pub const ANALYSIS_IC: u32 = 16384;
pub const ANALYSIS_STATIC: u32 = 32768;
pub const ANALYSIS_NODESET: u32 = 65536;
pub const EVAL_RET_FLAG_LIM: u32 = 1;
pub const EVAL_RET_FLAG_FATAL: u32 = 2;
pub const EVAL_RET_FLAG_FINISH: u32 = 4;
pub const EVAL_RET_FLAG_STOP: u32 = 8;
pub const LOG_LVL_MASK: u32 = 7;
pub const LOG_LVL_DEBUG: u32 = 0;
pub const LOG_LVL_DISPLAY: u32 = 1;
pub const LOG_LVL_INFO: u32 = 2;
pub const LOG_LVL_WARN: u32 = 3;
pub const LOG_LVL_ERR: u32 = 4;
pub const LOG_LVL_FATAL: u32 = 5;
pub const LOG_FMT_ERR: u32 = 16;
pub const INIT_ERR_OUT_OF_BOUNDS: u32 = 1;

pub struct OsdiLimFunction<'ll> {
    pub name: String,
    pub num_args: u32,
    pub func_ptr: &'ll llvm::Value,
}
impl<'ll> OsdiLimFunction<'ll> {
    pub fn to_ll_val(&self, ctx: &CodegenCx<'_, 'll>, tys: &'ll OsdiTys) -> &'ll llvm::Value {
        let fields = [
            ctx.const_str_uninterned(&self.name),
            ctx.const_unsigned_int(self.num_args),
            self.func_ptr,
        ];
        let ty = tys.osdi_lim_function;
        ctx.const_struct(ty, &fields)
    }
}
impl OsdiTyBuilder<'_, '_, '_> {
    fn osdi_lim_function(&mut self) {
        let ctx = self.ctx;
        let fields = [ctx.ty_ptr(), ctx.ty_int(), ctx.ty_ptr()];
        let ty = ctx.ty_struct("OsdiLimFunction", &fields);
        self.osdi_lim_function = Some(ty);
    }
}
impl OsdiTyBuilder<'_, '_, '_> {
    fn osdi_sim_paras(&mut self) {
        let ctx = self.ctx;
        let fields = [ctx.ty_ptr(), ctx.ty_ptr(), ctx.ty_ptr(), ctx.ty_ptr()];
        let ty = ctx.ty_struct("OsdiSimParas", &fields);
        self.osdi_sim_paras = Some(ty);
    }
}
impl OsdiTyBuilder<'_, '_, '_> {
    fn osdi_sim_info(&mut self) {
        let ctx = self.ctx;
        let fields = [
            self.osdi_sim_paras.unwrap(),
            ctx.ty_double(),
            ctx.ty_ptr(),
            ctx.ty_ptr(),
            ctx.ty_ptr(),
            ctx.ty_int(),
        ];
        let ty = ctx.ty_struct("OsdiSimInfo", &fields);
        self.osdi_sim_info = Some(ty);
    }
}
impl OsdiTyBuilder<'_, '_, '_> {
    fn osdi_init_error_payload(&mut self) {
        let ctx = self.ctx;
        unsafe {
            let align = [llvm::LLVMABIAlignmentOfType(self.target_data, ctx.ty_int())]
                .into_iter()
                .max()
                .unwrap();
            let mut size = [llvm::LLVMABISizeOfType(self.target_data, ctx.ty_int())]
                .into_iter()
                .max()
                .unwrap() as u32;
            size = (size + align - 1) / align;
            let elem = ctx.ty_aint(align * 8);
            let ty = ctx.ty_array(elem, size);
            self.osdi_init_error_payload = Some(ty);
        }
    }
}
impl OsdiTyBuilder<'_, '_, '_> {
    fn osdi_init_error(&mut self) {
        let ctx = self.ctx;
        let fields = [ctx.ty_int(), self.osdi_init_error_payload.unwrap()];
        let ty = ctx.ty_struct("OsdiInitError", &fields);
        self.osdi_init_error = Some(ty);
    }
}
impl OsdiTyBuilder<'_, '_, '_> {
    fn osdi_init_info(&mut self) {
        let ctx = self.ctx;
        let fields = [ctx.ty_int(), ctx.ty_int(), ctx.ty_ptr()];
        let ty = ctx.ty_struct("OsdiInitInfo", &fields);
        self.osdi_init_info = Some(ty);
    }
}
pub struct OsdiNodePair {
    pub node_1: u32,
    pub node_2: u32,
}
impl OsdiNodePair {
    pub fn to_ll_val<'ll>(&self, ctx: &CodegenCx<'_, 'll>, tys: &'ll OsdiTys) -> &'ll llvm::Value {
        let fields = [ctx.const_unsigned_int(self.node_1), ctx.const_unsigned_int(self.node_2)];
        let ty = tys.osdi_node_pair;
        ctx.const_struct(ty, &fields)
    }
}
impl OsdiTyBuilder<'_, '_, '_> {
    fn osdi_node_pair(&mut self) {
        let ctx = self.ctx;
        let fields = [ctx.ty_int(), ctx.ty_int()];
        let ty = ctx.ty_struct("OsdiNodePair", &fields);
        self.osdi_node_pair = Some(ty);
    }
}
pub struct OsdiJacobianEntry {
    pub nodes: OsdiNodePair,
    pub react_ptr_off: u32,
    pub flags: u32,
}
impl OsdiJacobianEntry {
    pub fn to_ll_val<'ll>(&self, ctx: &CodegenCx<'_, 'll>, tys: &'ll OsdiTys) -> &'ll llvm::Value {
        let fields = [
            self.nodes.to_ll_val(ctx, tys),
            ctx.const_unsigned_int(self.react_ptr_off),
            ctx.const_unsigned_int(self.flags),
        ];
        let ty = tys.osdi_jacobian_entry;
        ctx.const_struct(ty, &fields)
    }
}
impl OsdiTyBuilder<'_, '_, '_> {
    fn osdi_jacobian_entry(&mut self) {
        let ctx = self.ctx;
        let fields = [self.osdi_node_pair.unwrap(), ctx.ty_int(), ctx.ty_int()];
        let ty = ctx.ty_struct("OsdiJacobianEntry", &fields);
        self.osdi_jacobian_entry = Some(ty);
    }
}
pub struct OsdiNode {
    pub name: String,
    pub units: String,
    pub residual_units: String,
    pub resist_residual_off: u32,
    pub react_residual_off: u32,
    pub resist_limit_rhs_off: u32,
    pub react_limit_rhs_off: u32,
    pub is_flow: bool,
}
impl OsdiNode {
    pub fn to_ll_val<'ll>(&self, ctx: &CodegenCx<'_, 'll>, tys: &'ll OsdiTys) -> &'ll llvm::Value {
        let fields = [
            ctx.const_str_uninterned(&self.name),
            ctx.const_str_uninterned(&self.units),
            ctx.const_str_uninterned(&self.residual_units),
            ctx.const_unsigned_int(self.resist_residual_off),
            ctx.const_unsigned_int(self.react_residual_off),
            ctx.const_unsigned_int(self.resist_limit_rhs_off),
            ctx.const_unsigned_int(self.react_limit_rhs_off),
            ctx.const_c_bool(self.is_flow),
        ];
        let ty = tys.osdi_node;
        ctx.const_struct(ty, &fields)
    }
}
impl OsdiTyBuilder<'_, '_, '_> {
    fn osdi_node(&mut self) {
        let ctx = self.ctx;
        let fields = [
            ctx.ty_ptr(),
            ctx.ty_ptr(),
            ctx.ty_ptr(),
            ctx.ty_int(),
            ctx.ty_int(),
            ctx.ty_int(),
            ctx.ty_int(),
            ctx.ty_c_bool(),
        ];
        let ty = ctx.ty_struct("OsdiNode", &fields);
        self.osdi_node = Some(ty);
    }
}
pub struct OsdiParamOpvar {
    pub name: Vec<String>,
    pub num_alias: u32,
    pub description: String,
    pub units: String,
    pub flags: u32,
    pub len: u32,
}
impl OsdiParamOpvar {
    pub fn to_ll_val<'ll>(&self, ctx: &CodegenCx<'_, 'll>, tys: &'ll OsdiTys) -> &'ll llvm::Value {
        let arr_0: Vec<_> = self.name.iter().map(|it| ctx.const_str_uninterned(it)).collect();
        let fields = [
            ctx.const_arr_ptr(ctx.ty_ptr(), &arr_0),
            ctx.const_unsigned_int(self.num_alias),
            ctx.const_str_uninterned(&self.description),
            ctx.const_str_uninterned(&self.units),
            ctx.const_unsigned_int(self.flags),
            ctx.const_unsigned_int(self.len),
        ];
        let ty = tys.osdi_param_opvar;
        ctx.const_struct(ty, &fields)
    }
}
impl OsdiTyBuilder<'_, '_, '_> {
    fn osdi_param_opvar(&mut self) {
        let ctx = self.ctx;
        let fields =
            [ctx.ty_ptr(), ctx.ty_int(), ctx.ty_ptr(), ctx.ty_ptr(), ctx.ty_int(), ctx.ty_int()];
        let ty = ctx.ty_struct("OsdiParamOpvar", &fields);
        self.osdi_param_opvar = Some(ty);
    }
}
pub struct OsdiNoiseSource {
    pub name: String,
    pub nodes: OsdiNodePair,
}
impl OsdiNoiseSource {
    pub fn to_ll_val<'ll>(&self, ctx: &CodegenCx<'_, 'll>, tys: &'ll OsdiTys) -> &'ll llvm::Value {
        let fields = [ctx.const_str_uninterned(&self.name), self.nodes.to_ll_val(ctx, tys)];
        let ty = tys.osdi_noise_source;
        ctx.const_struct(ty, &fields)
    }
}
impl OsdiTyBuilder<'_, '_, '_> {
    fn osdi_noise_source(&mut self) {
        let ctx = self.ctx;
        let fields = [ctx.ty_ptr(), self.osdi_node_pair.unwrap()];
        let ty = ctx.ty_struct("OsdiNoiseSource", &fields);
        self.osdi_noise_source = Some(ty);
    }
}

// Defines order of descriptor entries
pub struct OsdiDescriptor<'ll> {
    pub name: String,
    pub num_nodes: u32,
    pub num_terminals: u32,
    pub nodes: Vec<OsdiNode>,
    pub num_jacobian_entries: u32,
    pub jacobian_entries: Vec<OsdiJacobianEntry>,
    pub num_collapsible: u32,
    pub collapsible: Vec<OsdiNodePair>,
    pub collapsed_offset: u32,
    pub noise_sources: Vec<OsdiNoiseSource>,
    pub num_noise_src: u32,
    pub num_params: u32,
    pub num_instance_params: u32,
    pub num_opvars: u32,
    pub param_opvar: Vec<OsdiParamOpvar>,
    pub node_mapping_offset: u32,
    pub jacobian_ptr_resist_offset: u32,
    pub num_states: u32,
    pub state_idx_off: u32,
    pub bound_step_offset: u32,
    pub instance_size: u32,
    pub model_size: u32,
    pub access: &'ll llvm::Value,
    pub setup_model: &'ll llvm::Value,
    pub setup_instance: &'ll llvm::Value,
    pub eval: &'ll llvm::Value,
    pub load_noise: &'ll llvm::Value,
    pub load_residual_resist: &'ll llvm::Value,
    pub load_residual_react: &'ll llvm::Value,
    pub load_limit_rhs_resist: &'ll llvm::Value,
    pub load_limit_rhs_react: &'ll llvm::Value,
    pub load_spice_rhs_dc: &'ll llvm::Value,
    pub load_spice_rhs_tran: &'ll llvm::Value,
    pub load_jacobian_resist: &'ll llvm::Value,
    pub load_jacobian_react: &'ll llvm::Value,
    pub load_jacobian_tran: &'ll llvm::Value,
    pub given_flag_model: &'ll llvm::Value,
    pub given_flag_instance: &'ll llvm::Value, 
    pub num_resistive_jacobian_entries: u32,
    pub num_reactive_jacobian_entries: u32,
    pub write_jacobian_array_resist: &'ll llvm::Value,
    pub write_jacobian_array_react: &'ll llvm::Value,
    pub num_inputs: u32, 
    pub inputs: Vec<OsdiNodePair>, 
    pub load_jacobian_with_offset_resist: &'ll llvm::Value,
    pub load_jacobian_with_offset_react: &'ll llvm::Value,
}
impl<'ll> OsdiDescriptor<'ll> {
    pub fn to_ll_val(&self, ctx: &CodegenCx<'_, 'll>, tys: &'ll OsdiTys) -> &'ll llvm::Value {
        let arr_3: Vec<_> = self.nodes.iter().map(|it| it.to_ll_val(ctx, tys)).collect();
        let arr_5: Vec<_> = self.jacobian_entries.iter().map(|it| it.to_ll_val(ctx, tys)).collect();
        let arr_7: Vec<_> = self.collapsible.iter().map(|it| it.to_ll_val(ctx, tys)).collect();
        let arr_9: Vec<_> = self.noise_sources.iter().map(|it| it.to_ll_val(ctx, tys)).collect();
        let arr_14: Vec<_> = self.param_opvar.iter().map(|it| it.to_ll_val(ctx, tys)).collect();
        let arr_inputs: Vec<_> = self.inputs.iter().map(|it| it.to_ll_val(ctx, tys)).collect();
        let fields = [
            ctx.const_str_uninterned(&self.name),
            ctx.const_unsigned_int(self.num_nodes),
            ctx.const_unsigned_int(self.num_terminals),
            ctx.const_arr_ptr(tys.osdi_node, &arr_3),
            ctx.const_unsigned_int(self.num_jacobian_entries),
            ctx.const_arr_ptr(tys.osdi_jacobian_entry, &arr_5),
            ctx.const_unsigned_int(self.num_collapsible),
            ctx.const_arr_ptr(tys.osdi_node_pair, &arr_7),
            ctx.const_unsigned_int(self.collapsed_offset),
            ctx.const_arr_ptr(tys.osdi_noise_source, &arr_9),
            ctx.const_unsigned_int(self.num_noise_src),
            ctx.const_unsigned_int(self.num_params),
            ctx.const_unsigned_int(self.num_instance_params),
            ctx.const_unsigned_int(self.num_opvars),
            ctx.const_arr_ptr(tys.osdi_param_opvar, &arr_14),
            ctx.const_unsigned_int(self.node_mapping_offset),
            ctx.const_unsigned_int(self.jacobian_ptr_resist_offset),
            ctx.const_unsigned_int(self.num_states),
            ctx.const_unsigned_int(self.state_idx_off),
            ctx.const_unsigned_int(self.bound_step_offset),
            ctx.const_unsigned_int(self.instance_size),
            ctx.const_unsigned_int(self.model_size),
            self.access,
            self.setup_model,
            self.setup_instance,
            self.eval,
            self.load_noise,
            self.load_residual_resist,
            self.load_residual_react,
            self.load_limit_rhs_resist,
            self.load_limit_rhs_react,
            self.load_spice_rhs_dc,
            self.load_spice_rhs_tran,
            self.load_jacobian_resist,
            self.load_jacobian_react,
            self.load_jacobian_tran, 
            self.given_flag_model, 
            self.given_flag_instance, 
            ctx.const_unsigned_int(self.num_resistive_jacobian_entries), 
            ctx.const_unsigned_int(self.num_reactive_jacobian_entries), 
            self.write_jacobian_array_resist,
            self.write_jacobian_array_react,
            ctx.const_unsigned_int(self.num_inputs),
            ctx.const_arr_ptr(tys.osdi_node_pair, &arr_inputs), 
            self.load_jacobian_with_offset_resist,
            self.load_jacobian_with_offset_react,
        ];
        let ty = tys.osdi_descriptor;
        ctx.const_struct(ty, &fields)
    }
}
impl OsdiTyBuilder<'_, '_, '_> {
    fn osdi_descriptor(&mut self) {
        let ctx = self.ctx;
        let fields = [
            ctx.ty_ptr(),
            ctx.ty_int(),
            ctx.ty_int(),
            ctx.ty_ptr(),
            ctx.ty_int(),
            ctx.ty_ptr(),
            ctx.ty_int(),
            ctx.ty_ptr(),
            ctx.ty_int(),
            ctx.ty_ptr(),
            ctx.ty_int(),
            ctx.ty_int(),
            ctx.ty_int(),
            ctx.ty_int(),
            ctx.ty_ptr(),
            ctx.ty_int(),
            ctx.ty_int(),
            ctx.ty_int(),
            ctx.ty_int(),
            ctx.ty_int(),
            ctx.ty_int(),
            ctx.ty_int(),
            ctx.ty_ptr(),
            ctx.ty_ptr(),
            ctx.ty_ptr(),
            ctx.ty_ptr(),
            ctx.ty_ptr(),
            ctx.ty_ptr(),
            ctx.ty_ptr(),
            ctx.ty_ptr(),
            ctx.ty_ptr(),
            ctx.ty_ptr(),
            ctx.ty_ptr(),
            ctx.ty_ptr(),
            ctx.ty_ptr(),
            ctx.ty_ptr(),
            // 0.3 ends here
            ctx.ty_ptr(), // given_flag_model()
            ctx.ty_ptr(), // given_flag_instance()
            ctx.ty_int(), // num_resistive_jacobian_entries
            ctx.ty_int(), // num_reactive_jacobian_entries
            ctx.ty_ptr(), // write_jacobian_array_resist()
            ctx.ty_ptr(), // write_jacobian_array_react()
            ctx.ty_int(), // num_inputs
            ctx.ty_ptr(), // inputs
        ];
        let ty = ctx.ty_struct("OsdiDescriptor", &fields);
        self.osdi_descriptor = Some(ty);
    }
}
#[derive(Clone)]
pub struct OsdiTys<'ll> {
    pub osdi_lim_function: &'ll llvm::Type,
    pub osdi_sim_paras: &'ll llvm::Type,
    pub osdi_sim_info: &'ll llvm::Type,
    pub osdi_init_error_payload: &'ll llvm::Type,
    pub osdi_init_error: &'ll llvm::Type,
    pub osdi_init_info: &'ll llvm::Type,
    pub osdi_node_pair: &'ll llvm::Type,
    pub osdi_jacobian_entry: &'ll llvm::Type,
    pub osdi_node: &'ll llvm::Type,
    pub osdi_param_opvar: &'ll llvm::Type,
    pub osdi_noise_source: &'ll llvm::Type,
    pub osdi_descriptor: &'ll llvm::Type,
}
impl<'ll> OsdiTys<'ll> {
    pub fn new(ctx: &CodegenCx<'_, 'll>, target_data: &llvm::TargetData) -> Self {
        let mut builder = OsdiTyBuilder {
            ctx,
            target_data,
            osdi_lim_function: None,
            osdi_sim_paras: None,
            osdi_sim_info: None,
            osdi_init_error_payload: None,
            osdi_init_error: None,
            osdi_init_info: None,
            osdi_node_pair: None,
            osdi_jacobian_entry: None,
            osdi_node: None,
            osdi_param_opvar: None,
            osdi_noise_source: None,
            osdi_descriptor: None,
        };
        builder.osdi_lim_function();
        builder.osdi_sim_paras();
        builder.osdi_sim_info();
        builder.osdi_init_error_payload();
        builder.osdi_init_error();
        builder.osdi_init_info();
        builder.osdi_node_pair();
        builder.osdi_jacobian_entry();
        builder.osdi_node();
        builder.osdi_param_opvar();
        builder.osdi_noise_source();
        builder.osdi_descriptor();
        builder.finish()
    }
}
struct OsdiTyBuilder<'a, 'b, 'll> {
    ctx: &'a CodegenCx<'b, 'll>,
    target_data: &'a llvm::TargetData,
    osdi_lim_function: Option<&'ll llvm::Type>,
    osdi_sim_paras: Option<&'ll llvm::Type>,
    osdi_sim_info: Option<&'ll llvm::Type>,
    osdi_init_error_payload: Option<&'ll llvm::Type>,
    osdi_init_error: Option<&'ll llvm::Type>,
    osdi_init_info: Option<&'ll llvm::Type>,
    osdi_node_pair: Option<&'ll llvm::Type>,
    osdi_jacobian_entry: Option<&'ll llvm::Type>,
    osdi_node: Option<&'ll llvm::Type>,
    osdi_param_opvar: Option<&'ll llvm::Type>,
    osdi_noise_source: Option<&'ll llvm::Type>,
    osdi_descriptor: Option<&'ll llvm::Type>,
}
impl<'ll> OsdiTyBuilder<'_, '_, 'll> {
    fn finish(self) -> OsdiTys<'ll> {
        OsdiTys {
            osdi_lim_function: self.osdi_lim_function.unwrap(),
            osdi_sim_paras: self.osdi_sim_paras.unwrap(),
            osdi_sim_info: self.osdi_sim_info.unwrap(),
            osdi_init_error_payload: self.osdi_init_error_payload.unwrap(),
            osdi_init_error: self.osdi_init_error.unwrap(),
            osdi_init_info: self.osdi_init_info.unwrap(),
            osdi_node_pair: self.osdi_node_pair.unwrap(),
            osdi_jacobian_entry: self.osdi_jacobian_entry.unwrap(),
            osdi_node: self.osdi_node.unwrap(),
            osdi_param_opvar: self.osdi_param_opvar.unwrap(),
            osdi_noise_source: self.osdi_noise_source.unwrap(),
            osdi_descriptor: self.osdi_descriptor.unwrap(),
        }
    }
}

/*
 * ******************************************************************************************
 * Copyright (c) 2019 Pascal Kuthe. This file is part of the VARF project.
 * It is subject to the license terms in the LICENSE file found in the top-level directory
 *  of this distribution and at  https://gitlab.com/DSPOM/VARF/blob/master/LICENSE.
 *  No part of VARF, including this file, may be copied, modified, propagated, or
 *  distributed except according to the terms contained in the LICENSE file.
 * *****************************************************************************************
 */

//! This module is responsible for lowering an [`Hir`](crate::hir::Hir) to an [`Mir`](crate::mir::Mir)
//!
//! This entails three main transformations
//!
//! * **Adding explicit type information** -
//!    Code generation generally requires explicit type information.
//!    This is represented in the MIR with distinct expression types for [real](crate::mir::RealExpression) and [integers](crate::mir::IntegerExpression).
//!    Most expressions have a distinct type input and output types. Those that do not are resolved based on type conversion rules
//!    Instead of being implicit type conversions are now distinct expressions that are added when required and legal
//!
//! * **folding constant expressions to values** -
//!    Constant expressions can generally not be evaluated as they may depend on parameters.
//!    However the constant expressions defining parameters may only depend on the default values of previously defined parameters.
//!    The evaluation of these expressions is done in the [`constant_eval`]  module.
//!
//! * **TypeChecking** -
//!     VerilogAMS only permits implicit type conversion under special circumstance and some operators are not defined for reals at all.
//!     During the other two transformations it is ensured that these rules are adhered (in fact without these rules the other transformation wouldn't be possible)
//!

use crate::ast;
use crate::hir::Hir;
use crate::hir_lowering::derivatives::DerivativeMap;
use crate::hir_lowering::error::{Error, Type, Warning};
use crate::ir::mir::{ExpressionId, Mir, Parameter, ParameterType};
use crate::ir::*;
use crate::ir::{Push, SafeRangeCreation};
use crate::mir::Attribute;
use crate::mir::*;
use crate::SourceMap;
use bitflags::_core::cell::RefCell;

pub mod control_flow;
pub mod derivatives;
pub mod error;

#[cfg(test)]
pub mod test;

struct HirToMirFold<'tag, 'lt> {
    pub errors: Vec<Error<'tag>>,
    pub warnings: Vec<Warning<'tag>>,
    hir: &'lt Hir<'tag>,
    mir: Box<Mir<'tag>>,
    variable_to_differentiate: DerivativeMap<'tag>,
}
impl<'tag, 'lt> HirToMirFold<'tag, 'lt> {
    pub fn new(hir: &'lt mut Hir<'tag>) -> Self {
        Self {
            errors: Vec::with_capacity(32),
            warnings: Vec::with_capacity(32),
            mir: unsafe { Mir::partial_initalize(hir) },
            hir: &*hir,
            variable_to_differentiate: DerivativeMap::with_capacity_and_hasher(
                64,
                Default::default(),
            ),
        }
    }

    fn fold(mut self) -> (Result<Box<Mir<'tag>>, Vec<Error<'tag>>>, Vec<Warning<'tag>>) {
        for parameter in self.hir.full_range() {
            self.fold_parameter(parameter)
        }

        for variable in self.hir.full_range() {
            self.fold_variable(variable)
        }

        for attribute in SafeRangeCreation::<AttributeId<'tag>>::full_range(self.hir) {
            let value = self.hir[attribute]
                .value
                .and_then(|val| self.fold_expression(val, &mut Self::derivative_of_reference));
            self.mir.push(Attribute {
                name: self.hir[attribute].name,
                value,
            });
        }

        for module in SafeRangeCreation::<ModuleId<'tag>>::full_range(self.hir) {
            let res = self.hir[module].map_with(|old| {
                let (analog_cfg, analog_dtree) = self.fold_block_into_cfg(old.analog);
                Module {
                    name: old.name,
                    port_list: old.port_list,
                    parameter_list: old.parameter_list,
                    analog_cfg,
                    analog_dtree,
                }
            });
            self.mir.push(res);
        }

        (
            if self.errors.is_empty() {
                Ok(self.mir)
            } else {
                Err(self.errors)
            },
            self.warnings,
        )
    }

    /// folds a variable by foldings its default value (to a typed representation)
    fn fold_variable(&mut self, variable: VariableId<'tag>) {
        let variable_type = match self.hir[variable].contents.variable_type {
            ast::VariableType::REAL | ast::VariableType::REALTIME => {
                let default_value = self.hir[variable]
                    .contents
                    .default_value
                    .and_then(|expr| self.fold_read_only_real_expression(expr));
                VariableType::Real(default_value)
            }

            ast::VariableType::INTEGER | ast::VariableType::TIME => {
                let default_value =
                    if let Some(default_value) = self.hir[variable].contents.default_value {
                        match self.fold_read_only_expression(default_value) {
                            Some(ExpressionId::Integer(expr)) => Some(expr),

                            Some(ExpressionId::Real(real_expr)) => Some(self.mir.push(Node {
                                source: self.mir[real_expr].source,
                                contents: IntegerExpression::RealCast(real_expr),
                            })),
                            Some(ExpressionId::String(_)) => {
                                self.errors.push(Error {
                                    error_type: Type::ExpectedNumber,
                                    source: self.hir[default_value].source,
                                });
                                return;
                            }

                            None => return,
                        }
                    } else {
                        None
                    };
                VariableType::Integer(default_value)
            }
        };

        self.mir.write(
            variable,
            self.hir[variable].copy_with(|old| Variable {
                name: old.name,
                variable_type,
            }),
        );
    }

    /// folds a parameter by evaluating the default value and any range bounds
    /// # Safety
    /// This function HAS to be called for EVERY Parameter when folding an HIR to an MIR.
    /// Parameters are not initialized and when the MIR is dropped drop will be called on the parameters.
    /// This is UB since parameters do implement drop (they contain vectors)
    /// Internally this function is very carefully written such that `self.mir[parameter]` is always initialized even when an error occurs
    fn fold_parameter(&mut self, parameter: ParameterId<'tag>) {
        let parameter_type = match self.hir[parameter].contents.parameter_type {
            ast::ParameterType::String() => todo!("String parameters"),
            ast::ParameterType::Numerical {
                parameter_type,
                ref included_ranges,
                ref excluded_ranges,
            } => match parameter_type {
                ast::VariableType::INTEGER | ast::VariableType::TIME => {
                    if let Some(type_info) =
                        self.eval_parameter_type(parameter, included_ranges, excluded_ranges)
                    {
                        ParameterType::Integer {
                            included_ranges: type_info.0,
                            excluded_ranges: type_info.1,
                            default_value: type_info.2,
                        }
                    } else {
                        //dummy values in case of errors to ensure every parameter gets initalized to prevent UB during drop.
                        // This is okay since self.errors is not empty anymore and therefore self.mir won't be returned
                        ParameterType::Integer {
                            included_ranges: Vec::new(),
                            excluded_ranges: Vec::new(),
                            default_value: 0,
                        }
                    }
                }
                ast::VariableType::REAL | ast::VariableType::REALTIME => {
                    if let Some(type_info) =
                        self.eval_parameter_type(parameter, included_ranges, excluded_ranges)
                    {
                        ParameterType::Real {
                            included_ranges: type_info.0,
                            excluded_ranges: type_info.1,
                            default_value: type_info.2,
                        }
                    } else {
                        //dummy values in case of errors to ensure every parameter gets initalized to prevent UB during drop.
                        // This is okay since self.errors is not empty anymore and therefore self.mir won't be returned
                        ParameterType::Real {
                            included_ranges: Vec::new(),
                            excluded_ranges: Vec::new(),
                            default_value: 0.0,
                        }
                    }
                }
            },
        };

        unsafe {
            //This is save since we write to all parameters
            self.mir.write_unsafe(
                parameter,
                self.hir[parameter].map_with(|old| Parameter {
                    name: old.name,
                    parameter_type,
                }),
            )
        }
    }
}

mod constant_eval;
mod expression_semantic;

impl<'tag> Hir<'tag> {
    /// Folds an hir to an mir by adding and checking type information
    /// Returns any errors that occur
    pub fn lower(
        mut self: Box<Self>,
    ) -> (
        std::result::Result<Box<Mir<'tag>>, (Vec<Error<'tag>>, Box<Self>)>,
        Vec<Warning<'tag>>,
    ) {
        let (res, warnings) = HirToMirFold::new(&mut self).fold();
        (res.map_err(|errors| (errors, self)), warnings)
    }

    /// Folds an hir to an mir by adding and checking type information
    /// Prints any errors that occur
    pub fn lower_and_print_errors(
        mut self: Box<Self>,
        source_map: &SourceMap,
        translate_line: bool,
    ) -> Option<Box<Mir<'tag>>> {
        let (res, warnings) = HirToMirFold::new(&mut self).fold();
        for warning in warnings {
            warning.print(source_map, &self, translate_line)
        }
        res.map_err(|errors| {
            errors
                .into_iter()
                .for_each(|error| error.print(source_map, &self, translate_line))
        })
        .ok()
    }
}

use ahash::AHashMap;
use cfg::{
    smallvec, BasicBlock, CfgParam, Const, ControlFlowGraph, InstIdx, InstrDst, Instruction, Op,
    Operand, Phi, PhiIdx, Terminator,
};
use data_flow::lattice::{FlatSet, JoinSemiLattice, SparseFlatSetMap};
use data_flow::{direction, Analysis, AnalysisDomain, ResultsVisitorMut, SplitEdgeEffects};

use crate::ssa_constants::SsaConstants;
use crate::{ConstPlaces, EvalCtx};

#[derive(Debug, PartialEq)]
pub struct BasicBlockConstants {
    pub reachable: bool,
    pub constants: ConstPlaces,
    temporaries_changed: bool,
}

impl Clone for BasicBlockConstants {
    fn clone(&self) -> Self {
        Self {
            reachable: self.reachable,
            constants: self.constants.clone(),
            temporaries_changed: self.temporaries_changed,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.constants.clone_from(&source.constants);
        self.reachable = source.reachable;
        self.temporaries_changed = source.temporaries_changed
    }
}

impl JoinSemiLattice for BasicBlockConstants {
    fn join(&mut self, other: &Self) -> bool {
        debug_assert!(other.reachable);
        self.temporaries_changed.join(&other.temporaries_changed)
            | self.reachable.join(&true)
            | self.constants.join(&other.constants)
    }
}

pub struct ConditionalConstPropagation<'a> {
    pub ssa_consts: SsaConstants,
    pub known_params: &'a AHashMap<CfgParam, Const>,
}

impl<'a> AnalysisDomain for ConditionalConstPropagation<'a> {
    type Domain = BasicBlockConstants;
    type Direction = direction::Forward;
    const NAME: &'static str = "Const Propagation";

    fn bottom_value(&self, cfg: &ControlFlowGraph) -> Self::Domain {
        BasicBlockConstants {
            reachable: false,
            temporaries_changed: false,
            constants: SparseFlatSetMap::new_empty(cfg.next_place.into()),
        }
    }

    fn initialize_start_block(&self, _cfg: &ControlFlowGraph, state: &mut BasicBlockConstants) {
        state.reachable = true;
        // places are treated as globals (meaning don't have to be initialized before use)
        // Treat them as unreachable instead of unknown by default to avoid bugs
        state.constants.top_sets.insert_all();
    }
}

impl<'a> Analysis for ConditionalConstPropagation<'a> {
    fn init_block(&self, _cfg: &ControlFlowGraph, state: &mut Self::Domain) -> bool {
        state.temporaries_changed = false;
        // Ignore block if not reachable
        state.reachable
    }

    fn apply_phi_effect(
        &self,
        _cfg: &ControlFlowGraph,
        state: &mut BasicBlockConstants,
        phi: &Phi,
        _bb: BasicBlock,
        _idx: PhiIdx,
    ) {
        let res = phi.sources.iter().fold(FlatSet::Bottom, |mut dst, (_, local)| {
            self.ssa_consts.join_into(*local, &mut dst);
            dst
        });

        state.temporaries_changed |= self.ssa_consts.set(phi.dst, res);
    }

    /// Updates the current dataflow state with the effect of evaluating a statement.
    #[inline]
    fn apply_instr_effect(
        &self,
        _cfg: &ControlFlowGraph,
        state: &mut BasicBlockConstants,
        instr: &Instruction,
        _idx: InstIdx,
        _bb: BasicBlock,
    ) {
        let res = EvalCtx {
            const_places: &state.constants,
            ssa_consts: &self.ssa_consts,
            known_params: self.known_params,
        }
        .eval_op(instr.op, &instr.args, instr.src);

        match instr.dst {
            cfg::InstrDst::Local(local) => {
                state.temporaries_changed |= self.ssa_consts.set(local, res);
            }
            cfg::InstrDst::Place(dst) => {
                state.constants.set_flat_set(dst, res);
            }
            cfg::InstrDst::Ignore => (),
        }
    }

    #[inline(always)]
    fn apply_edge_effects(
        &self,
        _cfg: &ControlFlowGraph,
        _block: BasicBlock,
        state: &Self::Domain,
    ) -> bool {
        state.reachable
    }

    #[inline]
    fn apply_split_edge_effects(
        &self,
        _cfg: &ControlFlowGraph,
        _block: BasicBlock,
        discr: &Operand,
        state: &Self::Domain,
        edge_effects: &mut impl SplitEdgeEffects<Self::Domain>,
    ) {
        // only propagate along reachable paths
        if !state.reachable {
            edge_effects.apply(|_, _, _| false)
        } else {
            let discr = EvalCtx {
                const_places: &state.constants,
                ssa_consts: &self.ssa_consts,
                known_params: self.known_params,
            }
            .get_operand(discr);

            if let FlatSet::Elem(val) = discr {
                // dont propagate reachable into the edge that can not be reached from this block
                edge_effects.apply(|_, _, switch_edge| switch_edge == val.unwrap_bool())
            }
        }
    }
}

pub struct WriteBackConsts<'a, 'b>(pub &'a ConditionalConstPropagation<'b>);

impl<'a, 'b> ResultsVisitorMut for WriteBackConsts<'a, 'b> {
    type FlowState = BasicBlockConstants;

    #[inline]
    fn visit_terminator_before_effect(
        &mut self,
        state: &Self::FlowState,
        term: &mut Terminator,
        _block: BasicBlock,
    ) {
        if let Terminator::Split { ref mut condition, .. } = term {
            let res = EvalCtx {
                const_places: &state.constants,
                ssa_consts: &self.0.ssa_consts,
                known_params: self.0.known_params,
            }
            .get_operand(condition);
            if let FlatSet::Elem(val) = res {
                *condition = Operand::Const(val)
            }
        }
    }

    #[inline]
    fn visit_instruction_after_effect(
        &mut self,
        state: &Self::FlowState,
        instr: &mut Instruction,
        _block: BasicBlock,
        _id: InstIdx,
    ) {
        let val = match instr.dst {
            InstrDst::Local(local) => self.0.ssa_consts.get_option(local),
            InstrDst::Place(place) => state.constants.element_sets.get(&place).cloned(),
            InstrDst::Ignore => None,
        };

        match val {
            Some(val) => {
                instr.op = Op::Copy;
                instr.args = smallvec![Operand::Const(val)]
            }
            None => {
                let eval_ctx = EvalCtx {
                    const_places: &state.constants,
                    ssa_consts: &self.0.ssa_consts,
                    known_params: self.0.known_params,
                };
                instr.visit_operands_mut(|arg| {
                    if let FlatSet::Elem(val) = eval_ctx.get_operand(arg) {
                        *arg = Operand::Const(val)
                    }
                });

                // some float operations form identities when one of their arguments is known
                // unless full fastmath flags are set (which we really shouldn't do) LLVM will not
                // assume these identities due to NAN/inf handeling
                // however for our purposes applying these optimizations is fine for derivative since this is auto generated code anyway
                // while this is not in any useful to the rest of openvaf this allows LLVM to perform more
                // optimizations and is therefore worthwile

                if instr.src < 0 {
                    let float_identity = match instr.op {
                        Op::RealSub
                            if instr.args[0] == 0f64.into() || instr.args[0] == (-0f64).into() =>
                        {
                            Some((1, Op::RealArtihNeg))
                        }
                        Op::RealAdd if instr.args[0] == 0f64.into() => Some((1, Op::Copy)),
                        Op::RealAdd | Op::RealSub
                            if instr.args[1] == 0f64.into() || instr.args[1] == (-0f64).into() =>
                        {
                            Some((0, Op::Copy))
                        }
                        Op::RealMul | Op::RealDiv if instr.args[1] == 1f64.into() => {
                            Some((0, Op::Copy))
                        }

                        Op::RealMul if instr.args[0] == 1f64.into() => Some((1, Op::Copy)),
                        _ => None,
                    };

                    if let Some((val, op)) = float_identity {
                        instr.op = op;
                        instr.args = smallvec![instr.args[val]]
                    }
                }
            }
        }
    }
}

use openvaf_middle::cfg::{
    CfgPass, ControlFlowGraph, IntLocation, InternedLocations, PhiData, TerminatorKind,
};
use openvaf_middle::{impl_pass_span, COperand, CallType, OperandData, RValue, StmntKind};

pub struct Visit<'a, V> {
    pub visitor: &'a mut V,
    pub locations: &'a InternedLocations,
}

impl<'a, C, V> CfgPass<'_, C> for Visit<'a, V>
where
    C: CallType,
    V: CfgVisitor<C>,
{
    type Result = ();

    fn run(self, cfg: &mut ControlFlowGraph<C>) -> Self::Result {
        for (bb, locations) in cfg.blocks.iter().zip(&self.locations.blocks) {
            let mut loc = locations.stmnt_start;
            for (stmnt, _) in &bb.statements {
                self.visitor.visit_stmnt(stmnt, loc, cfg);
                loc += 1;
            }
            if let Some(term) = &bb.terminator {
                if let TerminatorKind::Split { ref condition, .. } = term.kind {
                    self.visitor
                        .visit_rvalue(condition, locations.terminator, cfg)
                }
            }
            // TODO phi stmnts?
        }
    }

    impl_pass_span!("cfg_visit");
}

pub trait CfgVisitor<C: CallType> {
    fn visit_input(
        &mut self,
        _input: &<C as CallType>::I,
        _loc: IntLocation,
        _cfg: &ControlFlowGraph<C>,
    ) {
    }

    fn visit_operand(
        &mut self,
        operand: &COperand<C>,
        loc: IntLocation,
        cfg: &ControlFlowGraph<C>,
    ) {
        if let OperandData::Read(ref input) = operand.contents {
            self.visit_input(input, loc, cfg)
        }
    }

    fn visit_rvalue(&mut self, rval: &RValue<C>, loc: IntLocation, cfg: &ControlFlowGraph<C>) {
        for operand in rval.operands() {
            self.visit_operand(operand, loc, cfg)
        }
    }

    fn visit_stmnt(&mut self, stmnt: &StmntKind<C>, loc: IntLocation, cfg: &ControlFlowGraph<C>) {
        match *stmnt {
            StmntKind::Assignment(_, ref val) => self.visit_rvalue(val, loc, cfg),
            StmntKind::Call(_, ref args, _) => {
                for arg in args {
                    self.visit_operand(arg, loc, cfg)
                }
            }
            StmntKind::NoOp => {}
        }
    }

    fn visit_phi(&mut self, _phi: &PhiData, _loc: IntLocation, _cfg: &ControlFlowGraph<C>) {}
}
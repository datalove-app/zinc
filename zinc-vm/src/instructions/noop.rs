use crate::core::{RuntimeError, VMInstruction, VirtualMachine};
use algebra::Field;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::instructions::NoOperation;

impl<F, CS> VMInstruction<F, CS> for NoOperation
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn execute(&self, _vm: &mut VirtualMachine<F, CS>) -> Result<(), RuntimeError> {
        Ok(())
    }
}

use crate::core::{RuntimeError, VMInstruction, VirtualMachine};
use crate::Engine;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::instructions::NoOperation;

impl<E, CS> VMInstruction<E, CS> for NoOperation
where
    E: Engine,
    CS: ConstraintSystem<E::Fr>,
{
    fn execute(&self, _vm: &mut VirtualMachine<E, CS>) -> Result<(), RuntimeError> {
        Ok(())
    }
}

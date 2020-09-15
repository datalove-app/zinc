use crate::core::{InternalVM, VMInstruction};
use crate::core::{RuntimeError, VirtualMachine};
use algebra::Field;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::instructions::LoadSequence;

impl<F, CS> VMInstruction<F, CS> for LoadSequence
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn execute(&self, vm: &mut VirtualMachine<F, CS>) -> Result<(), RuntimeError> {
        for i in 0..self.len {
            let value = vm.load(self.address + i)?;
            vm.push(value)?;
        }

        Ok(())
    }
}

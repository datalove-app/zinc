use crate::core::{InternalVM, VMInstruction};
use crate::core::{RuntimeError, VirtualMachine};
use algebra::Field;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::LoadSequenceGlobal;

impl<F, CS> VMInstruction<F, CS> for LoadSequenceGlobal
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn execute(&self, vm: &mut VirtualMachine<F, CS>) -> Result<(), RuntimeError> {
        for i in 0..self.len {
            let value = vm.load_global(self.address + self.len - i - 1)?;
            vm.push(value)?;
        }

        Ok(())
    }
}

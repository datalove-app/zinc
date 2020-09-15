use crate::core::{InternalVM, VMInstruction, VirtualMachine};
use crate::Result;
use algebra::Field;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::instructions::Tee;

impl<F, CS> VMInstruction<F, CS> for Tee
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn execute(&self, vm: &mut VirtualMachine<F, CS>) -> Result {
        let value = vm.pop()?;
        vm.push(value.clone())?;
        vm.push(value)?;

        Ok(())
    }
}

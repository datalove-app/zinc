use crate::core::{InternalVM, VMInstruction};
use crate::core::{RuntimeError, VirtualMachine};
use algebra::Field;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::instructions::Swap;

impl<F, CS> VMInstruction<F, CS> for Swap
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn execute(&self, vm: &mut VirtualMachine<F, CS>) -> Result<(), RuntimeError> {
        let a = vm.pop()?;
        let b = vm.pop()?;
        vm.push(a)?;
        vm.push(b)
    }
}

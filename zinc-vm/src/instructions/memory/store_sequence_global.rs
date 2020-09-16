use crate::core::{InternalVM, VMInstruction};
use crate::core::{RuntimeError, VirtualMachine};
use crate::Engine;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::instructions::StoreSequenceGlobal;

impl<E, CS> VMInstruction<E, CS> for StoreSequenceGlobal
where
    E: Engine,
    CS: ConstraintSystem<E::Fr>,
{
    fn execute(&self, vm: &mut VirtualMachine<E, CS>) -> Result<(), RuntimeError> {
        for i in 0..self.len {
            let value = vm.pop()?;
            vm.store_global(self.address + i, value)?;
        }

        Ok(())
    }
}

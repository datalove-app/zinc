use crate::core::{Cell, InternalVM, VMInstruction};
use crate::core::{RuntimeError, VirtualMachine};
use algebra::Field;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::LoadByIndexGlobal;

impl<F, CS> VMInstruction<F, CS> for LoadByIndexGlobal
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn execute(&self, vm: &mut VirtualMachine<F, CS>) -> Result<(), RuntimeError> {
        let index = vm.pop()?.value()?;

        let mut array = Vec::new();
        for i in 0..self.len {
            array.push(vm.load_global(self.address + i)?.value()?);
        }

        let condition = vm.condition_top()?;
        let value = vm
            .operations()
            .conditional_array_get(&condition, array.as_slice(), &index)?;
        vm.push(Cell::Value(value))
    }
}

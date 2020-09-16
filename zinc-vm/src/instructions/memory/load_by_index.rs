use crate::core::{Cell, InternalVM, RuntimeError, VirtualMachine, VMInstruction};
use crate::Engine;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::instructions::LoadByIndex;

impl<E, CS> VMInstruction<E, CS> for LoadByIndex
where
    E: Engine,
    CS: ConstraintSystem<E::Fr>,
{
    fn execute(&self, vm: &mut VirtualMachine<E, CS>) -> Result<(), RuntimeError> {
        let index = vm.pop()?.value()?;

        let mut array = Vec::new();
        for i in 0..self.len {
            array.push(vm.load(self.address + i)?.value()?);
        }

        let condition = vm.condition_top()?;
        let value = vm
            .operations()
            .conditional_array_get(&condition, array.as_slice(), &index)?;
        vm.push(Cell::Value(value))
    }
}

#[cfg(test)]
mod tests {
    use crate::instructions::testing_utils::{TestingError, VMTestRunner};
    use zinc_bytecode::instructions::*;
    use zinc_bytecode::scalar::ScalarType;

    #[test]
    fn test_load_by_index() -> Result<(), TestingError> {
        VMTestRunner::new()
            .add(PushConst::new(0.into(), ScalarType::Field))
            .add(Store::new(0))
            .add(PushConst::new(1.into(), ScalarType::Field))
            .add(Store::new(1))
            .add(PushConst::new(2.into(), ScalarType::Field))
            .add(Store::new(2))
            .add(PushConst::new(3.into(), ScalarType::Field))
            .add(Store::new(3))
            .add(PushConst::new(1.into(), ScalarType::Field))
            .add(LoadByIndex::new(0, 4))
            .test(&[1])
    }
}

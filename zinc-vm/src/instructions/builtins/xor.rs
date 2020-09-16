use crate::core::{Cell, InternalVM, RuntimeError, VirtualMachine, VMInstruction};
use crate::Engine;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::instructions::Xor;

impl<E, CS> VMInstruction<E, CS> for Xor
where
    E: Engine,
    CS: ConstraintSystem<E::Fr>,
{
    fn execute(&self, vm: &mut VirtualMachine<E, CS>) -> Result<(), RuntimeError> {
        let right = vm.pop()?.value()?;
        let left = vm.pop()?.value()?;

        let xor = vm.operations().xor(left, right)?;

        vm.push(Cell::Value(xor))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::testing_utils::{TestingError, VMTestRunner};
    use zinc_bytecode::scalar::ScalarType;
    use zinc_bytecode::*;

    #[test]
    fn test_xor() -> Result<(), TestingError> {
        VMTestRunner::new()
            .add(PushConst::new(0.into(), ScalarType::Boolean))
            .add(PushConst::new(0.into(), ScalarType::Boolean))
            .add(Xor)
            .add(PushConst::new(0.into(), ScalarType::Boolean))
            .add(PushConst::new(1.into(), ScalarType::Boolean))
            .add(Xor)
            .add(PushConst::new(1.into(), ScalarType::Boolean))
            .add(PushConst::new(0.into(), ScalarType::Boolean))
            .add(Xor)
            .add(PushConst::new(1.into(), ScalarType::Boolean))
            .add(PushConst::new(1.into(), ScalarType::Boolean))
            .add(Xor)
            .test(&[0, 1, 1, 0])
    }
}

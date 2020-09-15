use crate::core::{Cell, InternalVM, VMInstruction};
use crate::core::{RuntimeError, VirtualMachine};
use algebra::Field;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::instructions::And;

impl<F, CS> VMInstruction<F, CS> for And
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn execute(&self, vm: &mut VirtualMachine<F, CS>) -> Result<(), RuntimeError> {
        let right = vm.pop()?.value()?;
        let left = vm.pop()?.value()?;

        let and = vm.operations().and(left, right)?;

        vm.push(Cell::Value(and))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gadgets::ScalarType;
    use crate::instructions::testing_utils::{TestingError, VMTestRunner};
    use zinc_bytecode::*;

    #[test]
    fn test_and() -> Result<(), TestingError> {
        VMTestRunner::new()
            .add(PushConst::new(0.into(), ScalarType::Boolean))
            .add(PushConst::new(0.into(), ScalarType::Boolean))
            .add(And)
            .add(PushConst::new(0.into(), ScalarType::Boolean))
            .add(PushConst::new(1.into(), ScalarType::Boolean))
            .add(And)
            .add(PushConst::new(1.into(), ScalarType::Boolean))
            .add(PushConst::new(0.into(), ScalarType::Boolean))
            .add(And)
            .add(PushConst::new(1.into(), ScalarType::Boolean))
            .add(PushConst::new(1.into(), ScalarType::Boolean))
            .add(And)
            .test(&[1, 0, 0, 0])
    }
}

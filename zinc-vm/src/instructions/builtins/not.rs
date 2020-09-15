use crate::core::{Cell, InternalVM, RuntimeError, VMInstruction, VirtualMachine};
use crate::gadgets;
use algebra::Field;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::instructions::Not;

impl<F, CS> VMInstruction<F, CS> for Not
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn execute(&self, vm: &mut VirtualMachine<F, CS>) -> Result<(), RuntimeError> {
        let value = vm.pop()?.value()?;

        let cs = vm.constraint_system();
        let not = gadgets::not(cs.ns(|| "not"), &value)?;

        vm.push(Cell::Value(not))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::instructions::testing_utils::{TestingError, VMTestRunner};
    use zinc_bytecode::scalar::ScalarType;
    use zinc_bytecode::*;

    #[test]
    fn test_not() -> Result<(), TestingError> {
        VMTestRunner::new()
            .add(PushConst::new(0.into(), ScalarType::Boolean))
            .add(Not)
            .add(PushConst::new(1.into(), ScalarType::Boolean))
            .add(Not)
            .test(&[0, 1])
    }
}

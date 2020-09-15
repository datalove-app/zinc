use crate::core::{Cell, InternalVM, VMInstruction};
use crate::core::{RuntimeError, VirtualMachine};
use algebra::Field;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::instructions::Eq;

impl<F, CS> VMInstruction<F, CS> for Eq
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn execute(&self, vm: &mut VirtualMachine<F, CS>) -> Result<(), RuntimeError> {
        let right = vm.pop()?.value()?;
        let left = vm.pop()?.value()?;

        let eq = vm.operations().eq(left, right)?;

        vm.push(Cell::Value(eq))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::instructions::testing_utils::{TestingError, VMTestRunner};
    use zinc_bytecode::*;

    #[test]
    fn test_eq() -> Result<(), TestingError> {
        VMTestRunner::new()
            .add(PushConst::new_field(1.into()))
            .add(PushConst::new_field(2.into()))
            .add(Eq)
            .add(PushConst::new_field(2.into()))
            .add(PushConst::new_field(2.into()))
            .add(Eq)
            .add(PushConst::new_field(2.into()))
            .add(PushConst::new_field(1.into()))
            .add(Eq)
            .test(&[0, 1, 0])
    }
}

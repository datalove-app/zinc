use crate::core::{Cell, InternalVM, VMInstruction};
use crate::core::{RuntimeError, VirtualMachine};
use crate::gadgets;
use algebra::Field;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::instructions::Ge;

impl<F, CS> VMInstruction<F, CS> for Ge
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn execute(&self, vm: &mut VirtualMachine<F, CS>) -> Result<(), RuntimeError> {
        let right = vm.pop()?.value()?;
        let left = vm.pop()?.value()?;

        let cs = vm.constraint_system();
        let ge = gadgets::ge(cs.ns(|| "ge"), &left, &right)?;

        vm.push(Cell::Value(ge))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::instructions::testing_utils::{TestingError, VMTestRunner};
    use zinc_bytecode::*;

    #[test]
    fn test_ge() -> Result<(), TestingError> {
        VMTestRunner::new()
            .add(PushConst::new_field(2.into()))
            .add(PushConst::new_field(1.into()))
            .add(Ge)
            .add(PushConst::new_field(2.into()))
            .add(PushConst::new_field(2.into()))
            .add(Ge)
            .add(PushConst::new_field(1.into()))
            .add(PushConst::new_field(2.into()))
            .add(Ge)
            .test(&[0, 1, 1])
    }
}

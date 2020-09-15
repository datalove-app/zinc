use crate::auto_const;
use crate::core::{Cell, InternalVM, RuntimeError, VMInstruction, VirtualMachine};
use crate::gadgets::{self, auto_const::prelude::*, ScalarType, ScalarTypeExpectation};
use algebra::Field;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::instructions::Sub;

impl<F, CS> VMInstruction<F, CS> for Sub
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn execute(&self, vm: &mut VirtualMachine<F, CS>) -> Result<(), RuntimeError> {
        let right = vm.pop()?.value()?;
        let left = vm.pop()?.value()?;

        let diff_type = ScalarType::expect_same(left.get_type(), right.get_type())?;

        let condition = vm.condition_top()?;
        let cs = vm.constraint_system();

        let unchecked_diff =
            auto_const!(gadgets::arithmetic::sub, cs.ns(|| "diff"), &left, &right)?;

        let diff = gadgets::types::conditional_type_check(
            cs.ns(|| "type check"),
            &condition,
            &unchecked_diff,
            diff_type,
        )?;

        vm.push(Cell::Value(diff))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::instructions::testing_utils::{TestingError, VMTestRunner};
    use zinc_bytecode::*;

    #[test]
    fn test_sub() -> Result<(), TestingError> {
        VMTestRunner::new()
            .add(PushConst::new_field(2.into()))
            .add(PushConst::new_field(1.into()))
            .add(Sub)
            .test(&[1])
    }
}

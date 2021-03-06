use crate::auto_const;
use crate::core::{Cell, InternalVM, VMInstruction};
use crate::core::{RuntimeError, VirtualMachine};
use crate::gadgets;
use crate::gadgets::auto_const::prelude::*;
use crate::gadgets::{ScalarType, ScalarTypeExpectation};
use algebra::Field;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::instructions::Mul;

impl<F, CS> VMInstruction<F, CS> for Mul
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn execute(&self, vm: &mut VirtualMachine<F, CS>) -> Result<(), RuntimeError> {
        let right = vm.pop()?.value()?;
        let left = vm.pop()?.value()?;

        let mul_type = ScalarType::expect_same(left.get_type(), right.get_type())?;

        let condition = vm.condition_top()?;
        let cs = vm.constraint_system();

        let unchecked_mul = auto_const!(gadgets::arithmetic::mul, cs.ns(|| "mul"), &left, &right)?;

        let mul = gadgets::types::conditional_type_check(
            cs.ns(|| "type check"),
            &condition,
            &unchecked_mul,
            mul_type,
        )?;

        vm.push(Cell::Value(mul))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::instructions::testing_utils::{TestingError, VMTestRunner};
    use zinc_bytecode::*;

    #[test]
    fn test_mul() -> Result<(), TestingError> {
        VMTestRunner::new()
            .add(PushConst::new_field(3.into()))
            .add(PushConst::new_field(4.into()))
            .add(Mul)
            .test(&[12])
    }
}

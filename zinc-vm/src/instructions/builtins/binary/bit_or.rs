use crate::core::{InternalVM, VMInstruction, VirtualMachine};
use crate::gadgets::utils::{bigint_to_fr, fr_to_bigint};
use crate::gadgets::{Scalar, ScalarType, ScalarTypeExpectation};
use crate::{Result, RuntimeError};
use algebra::Field;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::instructions::BitOr;

impl<F, CS> VMInstruction<F, CS> for BitOr
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn execute(&self, vm: &mut VirtualMachine<F, CS>) -> Result {
        let right = vm.pop()?.value()?;
        let left = vm.pop()?.value()?;

        let scalar_type = ScalarType::expect_same(left.get_type(), right.get_type())?;

        let left_value = fr_to_bigint(&left.get_constant()?, scalar_type.is_signed());
        let right_value = fr_to_bigint(&right.get_constant()?, scalar_type.is_signed());

        let result_value = &left_value | &right_value;

        let result_fr = bigint_to_fr::<F>(&result_value).ok_or(RuntimeError::ValueOverflow {
            value: result_value,
            scalar_type,
        })?;
        let result = Scalar::new_constant_fr(result_fr, scalar_type);
        vm.push(result.into())
    }
}

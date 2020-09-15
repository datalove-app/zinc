use crate::core::{InternalVM, VMInstruction, VirtualMachine};
use crate::gadgets::utils::{bigint_to_fr, fr_to_bigint};
use crate::gadgets::{Scalar, ScalarTypeExpectation};
use crate::{Result, RuntimeError};
use algebra::Field;
use num_bigint::BigInt;
use num_bigint::Sign;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::instructions::BitNot;

impl<F, CS> VMInstruction<F, CS> for BitNot
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn execute(&self, vm: &mut VirtualMachine<F, CS>) -> Result {
        let scalar = vm.pop()?.value()?;

        let scalar_type = scalar.get_type();
        let value = fr_to_bigint(&scalar.get_constant()?, scalar_type.is_signed());

        let mut mask = vec![0xFF; scalar_type.bit_length::<F>() / 8];
        if scalar_type.is_signed() {
            mask[0] = 0x7F;
        }

        let mut result_value = !value;
        result_value &= &BigInt::from_bytes_le(Sign::Plus, mask.as_slice());

        let result_fr = bigint_to_fr::<F>(&result_value).ok_or(RuntimeError::ValueOverflow {
            value: result_value,
            scalar_type,
        })?;
        let result = Scalar::new_constant_fr(result_fr, scalar_type);
        vm.push(result.into())
    }
}

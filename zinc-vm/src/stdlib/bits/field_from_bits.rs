use crate::core::EvaluationStack;
use crate::gadgets::{AllocatedNum, Scalar, ScalarType};
use crate::stdlib::NativeFunction;
use crate::Result;
use algebra::{Field, FpParameters, PrimeField};
use r1cs_core::ConstraintSystem;

pub struct FieldFromBits;

impl<F: PrimeField> NativeFunction<F> for FieldFromBits {
    fn execute<CS: ConstraintSystem<F>>(
        &self,
        mut cs: CS,
        stack: &mut EvaluationStack<F>,
    ) -> Result {
        let mut bits = Vec::with_capacity(F::Params::MODULUS_BITS as usize);
        for i in 0..F::Params::MODULUS_BITS {
            let bit = stack.pop()?.value()?;
            let boolean = bit.to_boolean(cs.ns(|| format!("to_boolean {}", i)))?;
            bits.push(boolean);
        }

        let num = AllocatedNum::pack_bits_to_element(cs.ns(|| "pack_bits_to_element"), &bits)?;

        stack.push(
            Scalar::new_unchecked_variable(num.get_value(), num.get_variable(), ScalarType::Field)
                .into(),
        )?;

        Ok(())
    }
}

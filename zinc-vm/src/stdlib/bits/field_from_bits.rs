use crate::core::EvaluationStack;
use crate::gadgets::{AllocatedNum, Scalar, ScalarType};
use crate::stdlib::NativeFunction;
use crate::{Engine, Result};
use algebra::{FpParameters, PrimeField};
use r1cs_core::ConstraintSystem;

pub struct FieldFromBits;

impl<E: Engine> NativeFunction<E> for FieldFromBits {
    fn execute<CS: ConstraintSystem<E::Fr>>(
        &self,
        mut cs: CS,
        stack: &mut EvaluationStack<E>,
    ) -> Result {
        let size = <<E as Engine>::Fr as PrimeField>::Params::MODULUS_BITS as usize;
        let mut bits = Vec::with_capacity(size);
        for i in 0..size {
            let bit = stack.pop()?.value()?;
            let boolean = bit.to_boolean(cs.ns(|| format!("to_boolean {}", i)))?;
            bits.push(boolean);
        }

        let num = AllocatedNum::<E>::pack_bits_to_element(cs.ns(|| "pack_bits_to_element"), &bits)?;

        stack.push(
            Scalar::<E>::new_unchecked_variable(
                num.get_value(),
                num.get_variable(),
                ScalarType::Field,
            )
            .into(),
        )?;

        Ok(())
    }
}

use crate::core::EvaluationStack;
use crate::gadgets::{AllocatedNum, Scalar};
use crate::stdlib::NativeFunction;
use crate::{Engine, MalformedBytecode, Result};
use algebra::{FpParameters, PrimeField};
use r1cs_core::ConstraintSystem;
use zinc_bytecode::scalar::IntegerType;

pub struct UnsignedFromBits {
    bit_length: usize,
}

impl UnsignedFromBits {
    pub fn new(inputs_count: usize) -> Self {
        Self {
            bit_length: inputs_count,
        }
    }
}

impl<E: Engine> NativeFunction<E> for UnsignedFromBits {
    fn execute<CS: ConstraintSystem<E::Fr>>(
        &self,
        mut cs: CS,
        stack: &mut EvaluationStack<E>,
    ) -> Result {
        if self.bit_length > <<E as Engine>::Fr as PrimeField>::Params::CAPACITY as usize {
            return Err(MalformedBytecode::InvalidArguments(format!(
                "unsigned_from_bits: integer type with length {} is not supported",
                self.bit_length
            ))
            .into());
        }

        let mut bits = Vec::with_capacity(self.bit_length);
        for i in 0..self.bit_length {
            let bit = stack.pop()?.value()?;
            let boolean = bit.to_boolean(cs.ns(|| format!("to_boolean {}", i)))?;
            bits.push(boolean);
        }

        let num = AllocatedNum::<E>::pack_bits_to_element(cs.ns(|| "pack_bits_to_element"), &bits)?;

        let int_type = IntegerType {
            is_signed: false,
            bitlength: self.bit_length,
        };

        let scalar = Scalar::<E>::new_unchecked_variable(
            num.get_value(),
            num.get_variable(),
            int_type.into(),
        );

        stack.push(scalar.into())?;

        Ok(())
    }
}

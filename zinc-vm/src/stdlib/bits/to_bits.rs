use crate::core::EvaluationStack;
use crate::gadgets::{self, IntegerType, Scalar, ScalarType};
use crate::stdlib::NativeFunction;
use crate::{Engine, Result, RuntimeError};
use num_bigint::BigInt;
use r1cs_core::ConstraintSystem;
use r1cs_std::bits::boolean::Boolean;

pub struct ToBits;

impl<E: Engine> NativeFunction<E> for ToBits {
    fn execute<CS: ConstraintSystem<E::Fr>>(
        &self,
        mut cs: CS,
        stack: &mut EvaluationStack<E>,
    ) -> Result {
        let scalar = stack.pop()?.value()?;
        let expr = scalar.to_expression::<CS>();

        let mut bits = match scalar.get_type() {
            ScalarType::Integer(t) => {
                if t.is_signed {
                    signed_to_bits(cs.ns(|| "signed_to_bits"), scalar)?
                } else {
                    expr.into_bits_le_fixed(cs.ns(|| "into_bits_le"), t.bitlength)?
                }
            }
            ScalarType::Boolean => vec![scalar.to_boolean(cs.ns(|| "to_boolean"))?],
            ScalarType::Field => {
                expr.into_bits_le_strict(cs.ns(|| "into_bits_le_strict"))?
            }
        };

        // We use big-endian
        bits.reverse();

        for bit in bits {
            let scalar = Scalar::new_unchecked_variable(
                bit.get_value_field::<E::Fr>(),
                bit.get_variable()
                    .expect("into_bits_le_fixed must allocate")
                    .get_variable(),
                ScalarType::Boolean,
            );
            stack.push(scalar.into())?;
        }

        Ok(())
    }
}

fn signed_to_bits<E, CS>(mut cs: CS, scalar: Scalar<E>) -> Result<Vec<Boolean>>
where
    E: Engine,
    CS: ConstraintSystem<E::Fr>,
{
    let bitlength = match scalar.get_type() {
        ScalarType::Integer(IntegerType {
            bitlength,
            is_signed: true,
        }) => bitlength,
        t => {
            return Err(RuntimeError::TypeError {
                expected: "signed type".to_string(),
                actual: t.to_string(),
            })
        }
    };

    let base_value = BigInt::from(1) << bitlength;
    let base = Scalar::new_constant_bigint(&base_value, ScalarType::Field)?;

    let complement = gadgets::add(cs.ns(|| "complement"), &scalar, &base)?;

    let bits = complement
        .to_expression::<CS>()
        .into_bits_le_fixed(cs.ns(|| "bits"), bitlength + 1)?;

    Ok(Vec::from(&bits[..bitlength]))
}

use crate::auto_const;
use crate::gadgets::auto_const::prelude::*;
use crate::gadgets::{Scalar, ScalarType};
use crate::{gadgets, Result};
use algebra::Field;
use r1cs_core::ConstraintSystem;
use zinc_bytecode::scalar::IntegerType;

pub fn abs<F, CS>(cs: CS, scalar: &Scalar<F>) -> Result<Scalar<F>>
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn inner<F, CS>(mut cs: CS, scalar: &Scalar<F>) -> Result<Scalar<F>>
    where
        F: Field,
        CS: ConstraintSystem<F>,
    {
        match scalar.get_type() {
            ScalarType::Field | ScalarType::Boolean => Ok(scalar.clone()),

            ScalarType::Integer(int_type) => {
                if !int_type.is_signed {
                    return Ok(scalar.clone());
                }

                let extended_type = IntegerType {
                    is_signed: true,
                    bitlength: int_type.bitlength + 1,
                }
                .into();

                let scalar = scalar.with_type_unchecked(extended_type);
                let zero = Scalar::new_constant_int(0, extended_type);
                let neg = gadgets::neg(cs.ns(|| "neg"), &scalar)?;
                let lt0 = gadgets::lt(cs.ns(|| "lt"), &scalar, &zero)?;
                gadgets::conditional_select(
                    cs.ns(|| "select"),
                    &lt0,
                    &neg.as_field(),
                    &scalar.as_field(),
                )
            }
        }
    }

    auto_const!(inner, cs, scalar)
}

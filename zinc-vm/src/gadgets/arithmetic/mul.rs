use crate::auto_const;
use crate::gadgets::auto_const::prelude::*;
use crate::gadgets::{Scalar, ScalarType};
use crate::Result;
use algebra::Field;
use r1cs_core::ConstraintSystem;

pub fn mul<F, CS>(cs: CS, left: &Scalar<F>, right: &Scalar<F>) -> Result<Scalar<F>>
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    pub fn mul_inner<F, CS>(mut cs: CS, left: &Scalar<F>, right: &Scalar<F>) -> Result<Scalar<F>>
    where
        F: Field,
        CS: ConstraintSystem<F>,
    {
        let mut value = None;

        let variable = cs.alloc(
            || "variable",
            || {
                let mut sum = left.grab_value()?;
                sum.mul_assign(&right.grab_value()?);
                value = Some(sum);
                Ok(sum)
            },
        )?;

        cs.enforce(
            || "constraint",
            |lc| lc + &left.lc::<CS>(),
            |lc| lc + &right.lc::<CS>(),
            |lc| lc + variable,
        );

        Ok(Scalar::new_unchecked_variable(
            value,
            variable,
            ScalarType::Field,
        ))
    }

    auto_const!(mul_inner, cs, left, right)
}

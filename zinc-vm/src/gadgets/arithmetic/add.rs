use crate::auto_const;
use crate::gadgets::auto_const::prelude::*;
use crate::gadgets::{Scalar, ScalarType};
use crate::{Engine, Result};
use r1cs_core::ConstraintSystem;
use std::ops::AddAssign;

pub fn add<E, CS>(cs: CS, left: &Scalar<E>, right: &Scalar<E>) -> Result<Scalar<E>>
where
    E: Engine,
    CS: ConstraintSystem<E::Fr>,
{
    fn add_inner<E, CS>(mut cs: CS, left: &Scalar<E>, right: &Scalar<E>) -> Result<Scalar<E>>
    where
        E: Engine,
        CS: ConstraintSystem<E::Fr>,
    {
        let mut value = None;

        let variable = cs.alloc(
            || "variable",
            || {
                let mut sum = left.grab_value()?;
                sum.add_assign(&right.grab_value()?);
                value = Some(sum);
                Ok(sum)
            },
        )?;

        cs.enforce(
            || "constraint",
            |lc| lc + &left.lc::<CS>() + &right.lc::<CS>(),
            |lc| lc + CS::one(),
            |lc| lc + variable,
        );

        Ok(Scalar::new_unchecked_variable(
            value,
            variable,
            ScalarType::Field,
        ))
    }

    auto_const!(add_inner, cs, left, right)
}

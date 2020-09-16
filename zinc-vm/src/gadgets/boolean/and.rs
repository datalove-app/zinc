use crate::auto_const;
use crate::gadgets::auto_const::prelude::*;
use crate::gadgets::{AllocatedNum, Scalar, ScalarType, ScalarTypeExpectation};
use crate::{Engine, Result};
use r1cs_core::ConstraintSystem;
use std::ops::MulAssign;

pub fn and<E, CS>(cs: CS, left: &Scalar<E>, right: &Scalar<E>) -> Result<Scalar<E>>
where
    E: Engine,
    CS: ConstraintSystem<E::Fr>,
{
    fn inner<E, CS>(mut cs: CS, left: &Scalar<E>, right: &Scalar<E>) -> Result<Scalar<E>>
    where
        E: Engine,
        CS: ConstraintSystem<E::Fr>,
    {
        left.get_type().assert_type(ScalarType::Boolean)?;
        right.get_type().assert_type(ScalarType::Boolean)?;

        let num = AllocatedNum::<E>::alloc(cs.ns(|| "value"), || {
            let mut conj = left.grab_value()?;
            conj.mul_assign(&right.grab_value()?);
            Ok(conj)
        })?;

        cs.enforce(
            || "equality",
            |lc| lc + &left.lc::<CS>(),
            |lc| lc + &right.lc::<CS>(),
            |lc| lc + num.get_variable(),
        );

        Ok(Scalar::new_unchecked_variable(
            num.get_value(),
            num.get_variable(),
            ScalarType::Boolean,
        ))
    }

    auto_const!(inner, cs, left, right)
}

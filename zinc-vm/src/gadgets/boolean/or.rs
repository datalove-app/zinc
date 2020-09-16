use crate::auto_const;
use crate::gadgets::auto_const::prelude::*;
use crate::gadgets::{AllocatedNum, Scalar, ScalarType, ScalarTypeExpectation};
use crate::{Engine, Result};
use algebra::{One, Zero};
use r1cs_core::ConstraintSystem;

pub fn or<E, CS>(cs: CS, left: &Scalar<E>, right: &Scalar<E>) -> Result<Scalar<E>>
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
            let l = left.grab_value()?;
            let r = right.grab_value()?;
            if l.is_zero() && r.is_zero() {
                Ok(E::Fr::zero())
            } else {
                Ok(E::Fr::one())
            }
        })?;

        cs.enforce(
            || "equality",
            |lc| lc + CS::one() - &left.lc::<CS>(),
            |lc| lc + CS::one() - &right.lc::<CS>(),
            |lc| lc + CS::one() - num.get_variable(),
        );

        Ok(Scalar::new_unchecked_variable(
            num.get_value(),
            num.get_variable(),
            ScalarType::Boolean,
        ))
    }

    auto_const!(inner, cs, left, right)
}

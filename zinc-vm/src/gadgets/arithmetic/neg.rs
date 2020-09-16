use crate::auto_const;
use crate::gadgets::auto_const::prelude::*;
use crate::gadgets::{Expression, Scalar};
use crate::{Engine, Result};
use r1cs_core::ConstraintSystem;

pub fn neg<E, CS>(cs: CS, scalar: &Scalar<E>) -> Result<Scalar<E>>
where
    E: Engine,
    CS: ConstraintSystem<E::Fr>,
{
    fn inner<E, CS>(mut cs: CS, scalar: &Scalar<E>) -> Result<Scalar<E>>
    where
        E: Engine,
        CS: ConstraintSystem<E::Fr>,
    {
        let expr = Expression::u64::<CS>(0) - scalar.to_expression::<CS>();
        let num = expr.into_number(cs.ns(|| "into_number"))?;
        Ok(num.into())
    }

    auto_const!(inner, cs, scalar)
}

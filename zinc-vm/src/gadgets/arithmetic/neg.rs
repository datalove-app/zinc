use crate::auto_const;
use crate::gadgets::auto_const::prelude::*;
use crate::gadgets::{Expression, Scalar};
use crate::Result;
use algebra::Field;
use r1cs_core::ConstraintSystem;

pub fn neg<F, CS>(cs: CS, scalar: &Scalar<F>) -> Result<Scalar<F>>
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn inner<F, CS>(mut cs: CS, scalar: &Scalar<F>) -> Result<Scalar<F>>
    where
        F: Field,
        CS: ConstraintSystem<F>,
    {
        let expr = Expression::u64::<CS>(0) - scalar.to_expression::<CS>();
        let num = expr.into_number(cs.ns(|| "into_number"))?;
        Ok(num.into())
    }

    auto_const!(inner, cs, scalar)
}

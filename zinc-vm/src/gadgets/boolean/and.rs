use crate::gadgets::auto_const::prelude::*;
use crate::gadgets::{AllocatedNum, Scalar, ScalarType, ScalarTypeExpectation};
use crate::{auto_const, Result};
use algebra::Field;
use r1cs_core::ConstraintSystem;
use r1cs_std::alloc::AllocGadget;

pub fn and<F, CS>(cs: CS, left: &Scalar<F>, right: &Scalar<F>) -> Result<Scalar<F>>
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn inner<F, CS>(mut cs: CS, left: &Scalar<F>, right: &Scalar<F>) -> Result<Scalar<F>>
    where
        F: Field,
        CS: ConstraintSystem<F>,
    {
        left.get_type().assert_type(ScalarType::Boolean)?;
        right.get_type().assert_type(ScalarType::Boolean)?;

        let num = AllocatedNum::alloc(cs.ns(|| "value"), || {
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

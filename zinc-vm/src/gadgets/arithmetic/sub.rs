use crate::auto_const;
use crate::gadgets::auto_const::prelude::*;
use crate::gadgets::{Scalar, ScalarType};
use crate::Result;
use algebra::Field;
use r1cs_core::ConstraintSystem;

pub fn sub<F, CS>(cs: CS, left: &Scalar<F>, right: &Scalar<F>) -> Result<Scalar<F>>
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    pub fn sub_inner<F, CS>(mut cs: CS, left: &Scalar<F>, right: &Scalar<F>) -> Result<Scalar<F>>
    where
        F: Field,
        CS: ConstraintSystem<F>,
    {
        let mut value = None;

        let variable = cs.alloc(
            || "variable",
            || {
                let mut tmp = left.grab_value()?;
                tmp.sub_assign(&right.grab_value()?);
                value = Some(tmp);
                Ok(tmp)
            },
        )?;

        cs.enforce(
            || "constraint",
            |lc| lc + &left.lc::<CS>() - &right.lc::<CS>(),
            |lc| lc + CS::one(),
            |lc| lc + variable,
        );

        Ok(Scalar::new_unchecked_variable(
            value,
            variable,
            ScalarType::Field,
        ))
    }

    auto_const!(sub_inner, cs, left, right)
}

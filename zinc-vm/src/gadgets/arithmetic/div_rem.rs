use crate::auto_const;
use crate::gadgets::auto_const::prelude::*;
use crate::gadgets::conditional_select::conditional_select;
use crate::gadgets::{utils, Scalar, ScalarType};
use crate::{gadgets, Result, RuntimeError};
use algebra::Field;
use r1cs_core::{ConstraintSystem, SynthesisError};
use r1cs_std::Assignment;
use zinc_utils::euclidean;

pub fn div_rem_conditional<F, CS>(
    mut cs: CS,
    condition: &Scalar<F>,
    left: &Scalar<F>,
    right: &Scalar<F>,
) -> Result<(Scalar<F>, Scalar<F>)>
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    let denom = conditional_select(
        cs.ns(|| "select denominator"),
        condition,
        right,
        &Scalar::new_constant_int(1, right.get_type()),
    )?;

    auto_const!(div_rem_enforce, cs, left, &denom)
}

/// This is enforcing that `right` is not zero.
pub fn div_rem_enforce<F, CS>(
    mut cs: CS,
    left: &Scalar<F>,
    right: &Scalar<F>,
) -> Result<(Scalar<F>, Scalar<F>)>
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    let nominator = left;
    let denominator = right;

    let mut quotient_value: Option<F> = None;
    let mut remainder_value: Option<F> = None;

    if let (Some(nom), Some(denom)) = (nominator.get_value(), denominator.get_value()) {
        let nom_bi = utils::fr_to_bigint(&nom, nominator.is_signed());
        let denom_bi = utils::fr_to_bigint(&denom, denominator.is_signed());

        let (q, r) = euclidean::div_rem(&nom_bi, &denom_bi).ok_or(RuntimeError::DivisionByZero)?;

        quotient_value = utils::bigint_to_fr::<F>(&q);
        remainder_value = utils::bigint_to_fr::<F>(&r);
    }

    let (quotient, remainder) = {
        let qutioent_var = cs.alloc(
            || "qutioent",
            || quotient_value.ok_or(SynthesisError::AssignmentMissing),
        )?;

        let remainder_var = cs.alloc(
            || "remainder",
            || remainder_value.ok_or(SynthesisError::AssignmentMissing),
        )?;

        cs.enforce(
            || "equality",
            |lc| lc + qutioent_var,
            |lc| lc + &denominator.lc::<CS>(),
            |lc| lc + &nominator.lc::<CS>() - remainder_var,
        );

        let quotient =
            Scalar::new_unchecked_variable(quotient_value, qutioent_var, ScalarType::Field);
        let remainder =
            Scalar::new_unchecked_variable(remainder_value, remainder_var, ScalarType::Field);

        (quotient, remainder)
    };

    let abs_denominator = gadgets::abs(cs.ns(|| "abs"), denominator)?;
    let lt = gadgets::lt(
        cs.ns(|| "lt"),
        &remainder.as_field(),
        &abs_denominator.as_field(),
    )?;
    let zero = Scalar::new_constant_int(0, remainder.get_type());
    let ge = gadgets::ge(cs.ns(|| "ge"), &remainder, &zero)?;
    cs.enforce(
        || "0 <= rem < |denominator|",
        |lc| lc + CS::one() - &lt.lc::<CS>(),
        |lc| lc + CS::one() - &ge.lc::<CS>(),
        |lc| lc,
    );

    Ok((quotient, remainder))
}

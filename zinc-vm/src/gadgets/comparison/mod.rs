use crate::gadgets::auto_const::prelude::*;
use crate::gadgets::{self, utils, AllocatedNum, Expression, Scalar, ScalarTypeExpectation};
use crate::{auto_const, Result, RuntimeError};
use algebra::{Field, PrimeField};
use num_bigint::BigInt;
use r1cs_core::{ConstraintSystem, SynthesisError};
use r1cs_std::bits::boolean::Boolean;
use zinc_bytecode::scalar::ScalarType;

pub fn gt<F, CS>(cs: CS, left: &Scalar<F>, right: &Scalar<F>) -> Result<Scalar<F>>
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    lt(cs, right, left)
}

pub fn ge<F, CS>(cs: CS, left: &Scalar<F>, right: &Scalar<F>) -> Result<Scalar<F>>
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    le(cs, right, left)
}

pub fn le<F, CS>(mut cs: CS, left: &Scalar<F>, right: &Scalar<F>) -> Result<Scalar<F>>
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    let is_gt = gt(cs.ns(|| "gt"), left, right)?;
    gadgets::not(cs.ns(|| "not"), &is_gt)
}

pub fn lt<F, CS>(cs: CS, left: &Scalar<F>, right: &Scalar<F>) -> Result<Scalar<F>>
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    pub fn less_than_inner<F, CS>(
        mut cs: CS,
        left: &Scalar<F>,
        right: &Scalar<F>,
    ) -> Result<Scalar<F>>
    where
        F: Field,
        CS: ConstraintSystem<F>,
    {
        let scalar_type = ScalarType::expect_same(left.get_type(), right.get_type())?;

        match scalar_type {
            ScalarType::Field => less_than_field(cs, left, right),
            ScalarType::Integer(int_type) => {
                let boolean = less_than_integer(
                    cs.ns(|| "less_than_integer"),
                    int_type.bitlength,
                    left,
                    right,
                )?;
                Scalar::from_boolean(cs.ns(|| "from_boolean"), boolean)
            }
            ScalarType::Boolean => Err(RuntimeError::TypeError {
                expected: "field or integer type".into(),
                actual: "boolean".to_string(),
            }),
        }
    }

    auto_const!(less_than_inner, cs, left, right)
}

fn less_than_field<F, CS>(mut cs: CS, left: &Scalar<F>, right: &Scalar<F>) -> Result<Scalar<F>>
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    let expr_a = left.to_expression::<CS>();
    let expr_b = right.to_expression::<CS>();

    let bits_a = expr_a.into_bits_le_strict(cs.ns(|| "a representation"))?;
    let bits_b = expr_b.into_bits_le_strict(cs.ns(|| "b representation"))?;

    let lower_bits_len: usize = F::NUM_BITS as usize / 2;
    let upper_bits_len: usize = F::NUM_BITS as usize - lower_bits_len;

    let a_lower =
        AllocatedNum::pack_bits_to_element(cs.ns(|| "a_lower"), &bits_a[..lower_bits_len])?;
    let b_lower =
        AllocatedNum::pack_bits_to_element(cs.ns(|| "b_lower"), &bits_b[..lower_bits_len])?;

    let a_upper =
        AllocatedNum::pack_bits_to_element(cs.ns(|| "a_upper"), &bits_a[lower_bits_len..])?;
    let b_upper =
        AllocatedNum::pack_bits_to_element(cs.ns(|| "b_upper"), &bits_b[lower_bits_len..])?;

    let upper_a_lt_b = less_than_integer(
        cs.ns(|| "upper_a_lt_b"),
        upper_bits_len,
        &a_upper.clone().into(),
        &b_upper.clone().into(),
    )?;

    let lower_a_lt_b = less_than_integer(
        cs.ns(|| "lower_a_lt_b"),
        lower_bits_len,
        &a_lower.into(),
        &b_lower.into(),
    )?;

    let upper_a_eq_b = AllocatedNum::equals(cs.ns(|| "upper_a_eq_b"), &a_upper, &b_upper)?;

    let lower_lt_and_upper_eq = Boolean::and(cs.ns(|| ""), &lower_a_lt_b, &upper_a_eq_b.into())?;

    let res = boolean_or(cs.ns(|| "lt"), &upper_a_lt_b, &lower_lt_and_upper_eq)?;
    Scalar::from_boolean(cs.ns(|| "from_boolean"), res)
}

fn less_than_integer<F, CS>(
    mut cs: CS,
    length: usize,
    left: &Scalar<F>,
    right: &Scalar<F>,
) -> Result<Boolean>
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    assert!(length < F::CAPACITY as usize);
    let base_bigint = (BigInt::from(1) << length) - BigInt::from(1);
    let base = utils::bigint_to_fr::<F>(&base_bigint).unwrap();

    let expr =
        Expression::constant::<CS>(base) - left.to_expression::<CS>() + right.to_expression::<CS>();
    let bits = expr.into_bits_le_fixed(cs.ns(|| "into_bits_le_fixed"), length + 1)?;

    Ok(bits.last().unwrap().clone())
}

fn boolean_or<F: Field, CS: ConstraintSystem<F>>(
    mut cs: CS,
    left: &Boolean,
    right: &Boolean,
) -> std::result::Result<Boolean, SynthesisError> {
    Ok(Boolean::and(cs.ns(|| "and"), &left.not(), &right.not())?.not())
}

pub fn eq<F, CS>(cs: CS, left: &Scalar<F>, right: &Scalar<F>) -> Result<Scalar<F>>
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    fn add_inner<F, CS>(mut cs: CS, left: &Scalar<F>, right: &Scalar<F>) -> Result<Scalar<F>>
    where
        F: Field,
        CS: ConstraintSystem<F>,
    {
        let le = left.to_expression::<CS>();
        let re = right.to_expression::<CS>();

        let eq = Expression::equals(cs.ns(|| "equals"), le, re)?;

        Scalar::from_boolean(cs.ns(|| "scalar"), Boolean::from(eq))
    }

    auto_const!(add_inner, cs, left, right)
}

pub fn ne<F, CS>(mut cs: CS, left: &Scalar<F>, right: &Scalar<F>) -> Result<Scalar<F>>
where
    F: Field,
    CS: ConstraintSystem<F>,
{
    let t = eq(cs.ns(|| "eq"), left, right)?;
    gadgets::not(cs.ns(|| "not"), &t)
}

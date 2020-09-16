mod multieq;
pub use multieq::*;

use crate::auto_const;
use crate::gadgets;
use crate::gadgets::auto_const::prelude::*;
use crate::gadgets::{utils, AllocatedNum, Expression, Scalar, ScalarTypeExpectation};
use crate::{Engine, Result, RuntimeError};
use algebra::{FpParameters, PrimeField};
use num_bigint::BigInt;
use r1cs_core::{ConstraintSystem, Namespace, SynthesisError};
use r1cs_std::prelude::Boolean;
use zinc_bytecode::scalar::ScalarType;

pub fn gt<E, CS>(cs: CS, left: &Scalar<E>, right: &Scalar<E>) -> Result<Scalar<E>>
where
    E: Engine,
    CS: ConstraintSystem<E::Fr>,
{
    lt(cs, right, left)
}

pub fn ge<E, CS>(cs: CS, left: &Scalar<E>, right: &Scalar<E>) -> Result<Scalar<E>>
where
    E: Engine,
    CS: ConstraintSystem<E::Fr>,
{
    le(cs, right, left)
}

pub fn le<E, CS>(mut cs: CS, left: &Scalar<E>, right: &Scalar<E>) -> Result<Scalar<E>>
where
    E: Engine,
    CS: ConstraintSystem<E::Fr>,
{
    let is_gt = gt(cs.ns(|| "gt"), left, right)?;
    gadgets::not(cs.ns(|| "not"), &is_gt)
}

pub fn lt<E, CS>(cs: CS, left: &Scalar<E>, right: &Scalar<E>) -> Result<Scalar<E>>
where
    E: Engine,
    CS: ConstraintSystem<E::Fr>,
{
    pub fn less_than_inner<E, CS>(
        mut cs: CS,
        left: &Scalar<E>,
        right: &Scalar<E>,
    ) -> Result<Scalar<E>>
    where
        E: Engine,
        CS: ConstraintSystem<E::Fr>,
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

pub fn eq<E, CS>(cs: CS, left: &Scalar<E>, right: &Scalar<E>) -> Result<Scalar<E>>
where
    E: Engine,
    CS: ConstraintSystem<E::Fr>,
{
    fn add_inner<E, CS>(mut cs: CS, left: &Scalar<E>, right: &Scalar<E>) -> Result<Scalar<E>>
    where
        E: Engine,
        CS: ConstraintSystem<E::Fr>,
    {
        let le = left.to_expression::<CS>();
        let re = right.to_expression::<CS>();

        let eq = Expression::equals(cs.ns(|| "equals"), le, re)?;

        Scalar::from_boolean(cs.ns(|| "scalar"), Boolean::from(eq))
    }

    auto_const!(add_inner, cs, left, right)
}

pub fn ne<E, CS>(mut cs: CS, left: &Scalar<E>, right: &Scalar<E>) -> Result<Scalar<E>>
where
    E: Engine,
    CS: ConstraintSystem<E::Fr>,
{
    let t = eq(cs.ns(|| "eq"), left, right)?;
    gadgets::not(cs.ns(|| "not"), &t)
}

fn less_than_field<E, CS>(mut cs: CS, left: &Scalar<E>, right: &Scalar<E>) -> Result<Scalar<E>>
where
    E: Engine,
    CS: ConstraintSystem<E::Fr>,
{
    let expr_a = left.to_expression::<CS>();
    let expr_b = right.to_expression::<CS>();

    let bits_a = expr_a.into_bits_le_strict(cs.ns(|| "a representation"))?;
    let bits_b = expr_b.into_bits_le_strict(cs.ns(|| "b representation"))?;

    let size = <<E as Engine>::Fr as PrimeField>::Params::MODULUS_BITS as usize;
    let lower_bits_len: usize = size / 2;
    let upper_bits_len: usize = size - lower_bits_len;

    let a_lower =
        AllocatedNum::<E>::pack_bits_to_element(cs.ns(|| "a_lower"), &bits_a[..lower_bits_len])?;
    let b_lower =
        AllocatedNum::<E>::pack_bits_to_element(cs.ns(|| "b_lower"), &bits_b[..lower_bits_len])?;

    let a_upper =
        AllocatedNum::<E>::pack_bits_to_element(cs.ns(|| "a_upper"), &bits_a[lower_bits_len..])?;
    let b_upper =
        AllocatedNum::<E>::pack_bits_to_element(cs.ns(|| "b_upper"), &bits_b[lower_bits_len..])?;

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

    let res = boolean_or::<E, Namespace<'_, E::Fr, CS::Root>>(
        cs.ns(|| "lt"),
        &upper_a_lt_b,
        &lower_lt_and_upper_eq,
    )?;
    Scalar::from_boolean(cs.ns(|| "from_boolean"), res)
}

fn less_than_integer<E, CS>(
    mut cs: CS,
    length: usize,
    left: &Scalar<E>,
    right: &Scalar<E>,
) -> Result<Boolean>
where
    E: Engine,
    CS: ConstraintSystem<E::Fr>,
{
    assert!(length < <<E as Engine>::Fr as PrimeField>::Params::CAPACITY as usize);
    let base_bigint = (BigInt::from(1) << length) - BigInt::from(1);
    let base = utils::bigint_to_fr::<E::Fr>(&base_bigint).unwrap();

    let expr =
        Expression::constant::<CS>(base) - left.to_expression::<CS>() + right.to_expression::<CS>();
    let bits = expr.into_bits_le_fixed(cs.ns(|| "into_bits_le_fixed"), length + 1)?;

    Ok(bits.last().unwrap().clone())
}

fn boolean_or<E, CS>(
    mut cs: CS,
    left: &Boolean,
    right: &Boolean,
) -> std::result::Result<Boolean, SynthesisError>
where
    E: Engine,
    CS: ConstraintSystem<E::Fr>,
{
    Ok(Boolean::and(cs.ns(|| "and"), &left.not(), &right.not())?.not())
}

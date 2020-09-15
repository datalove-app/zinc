use algebra::{BigInteger, Field, FpParameters, PrimeField};
use num_bigint::{BigInt, Sign};
use num_traits::Signed;
use std::{
    ops::{Div, Neg},
    str::FromStr,
};

pub fn fr_to_bigint<F: PrimeField>(fr: &F, signed: bool) -> BigInt {
    if signed {
        fr_to_bigint_signed(fr)
    } else {
        fr_to_bigint_unsigned(fr)
    }
}

pub fn fr_to_bigint_signed<F: PrimeField>(fr: &F) -> BigInt {
    let mut buffer = Vec::<u8>::new();
    F::Params::MODULUS
        .write_be(&mut buffer)
        .expect("failed to write into Vec<u8>");
    let modulus = BigInt::from_bytes_be(Sign::Plus, &buffer);
    buffer.clear();

    fr.into_repr()
        .write_be(&mut buffer)
        .expect("failed to write into Vec<u8>");
    let value = BigInt::from_bytes_be(Sign::Plus, &buffer);

    if value < (modulus.clone().div(2)) {
        value
    } else {
        value - modulus
    }
}

pub fn fr_to_bigint_unsigned<F: PrimeField>(fr: &F) -> BigInt {
    let mut buffer = Vec::<u8>::new();
    fr.into_repr()
        .write_le(&mut buffer)
        .expect("failed to write into Vec<u8>");
    BigInt::from_bytes_le(Sign::Plus, &buffer)
}

pub fn bigint_to_fr<F: PrimeField>(bigint: &BigInt) -> Option<F> {
    if bigint.is_positive() {
        let (_, bytes) = bigint.to_bytes_le();
        F::BigInt::from_bytes(&bytes).ok()
    } else {
        let (_, bytes) = bigint.neg().to_bytes_le();
        let abs = F::BigInt::from_bytes(&bytes).ok()?;
        let mut fr = F::zero();
        fr.sub_assign(&abs);
        Some(fr)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use algebra::jubjub::fields::Fr;
    use num_traits::ToPrimitive;
    use std::str::FromStr;

    #[test]
    fn test_fr_to_bigint() {
        let values = [0, 1, 2, 42, 1_234_567_890];

        for v in values.iter() {
            let fr = Fr::from_str(&v.to_string()).unwrap();
            let bigint = fr_to_bigint(&fr, true);
            assert_eq!(bigint.to_i32(), Some(*v));
        }
    }

    #[test]
    fn test_bigint_to_fr() {
        let values = [0, 1, 2, 42, 1_234_567_890];

        for &v in values.iter() {
            let bigint = BigInt::from(v);
            let fr = bigint_to_fr(&bigint);
            assert_eq!(fr, Fr::from_str(&v.to_string()));
        }
    }

    #[test]
    fn test_negatives() {
        let values = [-1 as isize, -42, -123_456_789_098_761];

        for &v in values.iter() {
            let expected = BigInt::from(v);
            let fr = bigint_to_fr(&expected).expect("bigint_to_fr");
            let actual = fr_to_bigint(&fr, true);
            assert_eq!(actual, expected);
        }
    }
}

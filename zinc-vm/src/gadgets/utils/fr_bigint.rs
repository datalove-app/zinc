use algebra::{BigInteger, FpParameters, PrimeField};
use num_bigint::{BigInt, Sign};
use num_traits::Signed;
use std::ops::{Div, Neg};

pub fn fr_to_bigint<Fr: PrimeField>(fr: &Fr, signed: bool) -> BigInt {
    if signed {
        fr_to_bigint_signed(fr)
    } else {
        fr_to_bigint_unsigned(fr)
    }
}

pub fn fr_to_bigint_signed<Fr: PrimeField>(fr: &Fr) -> BigInt {
    let mut buffer = Vec::<u8>::new();
    Fr::Params::MODULUS
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

pub fn fr_to_bigint_unsigned<Fr: PrimeField>(fr: &Fr) -> BigInt {
    let mut buffer = Vec::<u8>::new();
    fr.into_repr()
        .write_be(&mut buffer)
        .expect("failed to write into Vec<u8>");
    BigInt::from_bytes_be(Sign::Plus, &buffer)
}

pub fn bigint_to_fr<Fr: PrimeField>(bigint: &BigInt) -> Option<Fr> {
    if bigint.is_positive() {
        Fr::from_str(&bigint.to_str_radix(10)).ok()
    } else {
        let abs = Fr::from_str(&bigint.neg().to_str_radix(10)).ok()?;
        let mut fr = Fr::zero();
        fr.sub_assign(&abs);
        Some(fr)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bellman::pairing::bn256::{Bn256, Fr};
    use num_traits::ToPrimitive;

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
            let fr = bigint_to_fr::<Bn256>(&bigint);
            assert_eq!(fr, Fr::from_str(&v.to_string()));
        }
    }

    #[test]
    fn test_negatives() {
        let values = [-1 as isize, -42, -123_456_789_098_761];

        for &v in values.iter() {
            let expected = BigInt::from(v);
            let fr = bigint_to_fr::<Bn256>(&expected).expect("bigint_to_fr");
            let actual = fr_to_bigint(&fr, true);
            assert_eq!(actual, expected);
        }
    }
}

use crate::core::EvaluationStack;
use crate::errors::RuntimeError;
use crate::gadgets::{MultiEq, Scalar, UInt32};
use crate::stdlib::NativeFunction;
use crate::{Engine, MalformedBytecode};
use algebra::PrimeField;
use r1cs_core::{ConstraintSystem, Namespace, SynthesisError};
use r1cs_std::{
    alloc::AllocGadget,
    bits::boolean::{AllocatedBit, Boolean},
};

const ROUND_CONSTANTS: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

const IV: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

pub struct Sha256 {
    message_length: usize,
}

impl Sha256 {
    pub fn new(message_length: usize) -> Result<Self, RuntimeError> {
        if message_length % 8 == 0 {
            Ok(Self { message_length })
        } else {
            Err(MalformedBytecode::InvalidArguments(format!(
                "message length for sha256 must be a multiple of 8, got {}",
                message_length
            ))
            .into())
        }
    }

    fn run<F, CS>(mut cs: CS, input: &[Boolean]) -> Result<Vec<Boolean>, SynthesisError>
    where
        F: PrimeField,
        CS: ConstraintSystem<F>,
    {
        assert!(input.len() % 8 == 0);

        let mut padded = input.to_vec();
        let plen = padded.len() as u64;
        // append a single '1' bit
        padded.push(Boolean::constant(true));
        // append K '0' bits, where K is the minimum number >= 0 such that L + 1 + K + 64 is a multiple of 512
        while (padded.len() + 64) % 512 != 0 {
            padded.push(Boolean::constant(false));
        }
        // append L as a 64-bit big-endian integer, making the total post-processed length a multiple of 512 bits
        for b in (0..64).rev().map(|i| (plen >> i) & 1 == 1) {
            padded.push(Boolean::constant(b));
        }
        assert!(padded.len() % 512 == 0);

        let mut cur = Self::iv();
        for (i, block) in padded.chunks(512).enumerate() {
            cur = Self::compress(cs.ns(|| format!("block {}", i)), block, &cur)?;
        }

        Ok(cur.into_iter().flat_map(|e| e.to_bits_be()).collect())
    }

    fn compress<F, CS>(
        cs: CS,
        input: &[Boolean],
        current_hash_value: &[UInt32],
    ) -> Result<Vec<UInt32>, SynthesisError>
    where
        F: PrimeField,
        CS: ConstraintSystem<F>,
    {
        assert_eq!(input.len(), 512);
        assert_eq!(current_hash_value.len(), 8);

        let mut w: Vec<UInt32> = input.chunks(32).map(UInt32::from_bits_be).collect();

        // We can save some constraints by combining some of
        // the constraints in different u32 additions
        let mut cs = MultiEq::<F, CS>::from(cs);

        for i in 16..64 {
            let cs = &mut cs.ns(|| format!("w extension {}", i));

            // s0 := (w[i-15] rightrotate 7) xor (w[i-15] rightrotate 18) xor (w[i-15] rightshift 3)
            let mut s0 = w[i - 15].rotr(7);
            s0 = s0.xor(cs.ns(|| "first xor for s0"), &w[i - 15].rotr(18))?;
            s0 = s0.xor(cs.ns(|| "second xor for s0"), &w[i - 15].shr(3))?;

            // s1 := (w[i-2] rightrotate 17) xor (w[i-2] rightrotate 19) xor (w[i-2] rightshift 10)
            let mut s1 = w[i - 2].rotr(17);
            s1 = s1.xor(cs.ns(|| "first xor for s1"), &w[i - 2].rotr(19))?;
            s1 = s1.xor(cs.ns(|| "second xor for s1"), &w[i - 2].shr(10))?;

            let tmp = UInt32::addmany(
                cs.ns(|| "computation of w[i]"),
                &[w[i - 16].clone(), s0, w[i - 7].clone(), s1],
            )?;

            // w[i] := w[i-16] + s0 + w[i-7] + s1
            w.push(tmp);
        }

        assert_eq!(w.len(), 64);

        enum Maybe {
            Deferred(Vec<UInt32>),
            Concrete(UInt32),
        }

        impl Maybe {
            fn compute<F, CS, M>(self, cs: M, others: &[UInt32]) -> Result<UInt32, SynthesisError>
            where
                F: PrimeField,
                CS: ConstraintSystem<F>,
                M: ConstraintSystem<F, Root = MultiEq<F, CS>>,
            {
                Ok(match self {
                    Maybe::Concrete(ref v) => return Ok(v.clone()),
                    Maybe::Deferred(mut v) => {
                        v.extend(others.into_iter().cloned());
                        UInt32::addmany(cs, &v)?
                    }
                })
            }
        }

        let mut a = Maybe::Concrete(current_hash_value[0].clone());
        let mut b = current_hash_value[1].clone();
        let mut c = current_hash_value[2].clone();
        let mut d = current_hash_value[3].clone();
        let mut e = Maybe::Concrete(current_hash_value[4].clone());
        let mut f = current_hash_value[5].clone();
        let mut g = current_hash_value[6].clone();
        let mut h = current_hash_value[7].clone();

        for i in 0..64 {
            let cs = &mut cs.ns(|| format!("compression round {}", i));

            // S1 := (e rightrotate 6) xor (e rightrotate 11) xor (e rightrotate 25)
            let new_e = e.compute(cs.ns(|| "deferred e computation"), &[])?;
            let mut s1 = new_e.rotr(6);
            s1 = s1.xor(cs.ns(|| "first xor for s1"), &new_e.rotr(11))?;
            s1 = s1.xor(cs.ns(|| "second xor for s1"), &new_e.rotr(25))?;

            // ch := (e and f) xor ((not e) and g)
            let ch = UInt32::ch(cs.ns(|| "ch"), &new_e, &f, &g)?;

            // temp1 := h + S1 + ch + k[i] + w[i]
            let temp1 = vec![
                h.clone(),
                s1,
                ch,
                UInt32::constant(ROUND_CONSTANTS[i]),
                w[i].clone(),
            ];

            // S0 := (a rightrotate 2) xor (a rightrotate 13) xor (a rightrotate 22)
            let new_a = a.compute(cs.ns(|| "deferred a computation"), &[])?;
            let mut s0 = new_a.rotr(2);
            s0 = s0.xor(cs.ns(|| "first xor for s0"), &new_a.rotr(13))?;
            s0 = s0.xor(cs.ns(|| "second xor for s0"), &new_a.rotr(22))?;

            // maj := (a and b) xor (a and c) xor (b and c)
            let maj = UInt32::maj(cs.ns(|| "maj"), &new_a, &b, &c)?;

            // temp2 := S0 + maj
            let temp2 = vec![s0, maj];

            /*
            h := g
            g := f
            f := e
            e := d + temp1
            d := c
            c := b
            b := a
            a := temp1 + temp2
            */

            h = g;
            g = f;
            f = new_e;
            e = Maybe::Deferred(temp1.iter().cloned().chain(Some(d)).collect::<Vec<_>>());
            d = c;
            c = b;
            b = new_a;
            a = Maybe::Deferred(
                temp1
                    .iter()
                    .cloned()
                    .chain(temp2.iter().cloned())
                    .collect::<Vec<_>>(),
            );
        }

        /*
            Add the compressed chunk to the current hash value:
            h0 := h0 + a
            h1 := h1 + b
            h2 := h2 + c
            h3 := h3 + d
            h4 := h4 + e
            h5 := h5 + f
            h6 := h6 + g
            h7 := h7 + h
        */

        let h0 = a.compute(
            cs.ns(|| "deferred h0 computation"),
            &[current_hash_value[0].clone()],
        )?;

        let h1 = UInt32::addmany(cs.ns(|| "new h1"), &[current_hash_value[1].clone(), b])?;

        let h2 = UInt32::addmany(cs.ns(|| "new h2"), &[current_hash_value[2].clone(), c])?;

        let h3 = UInt32::addmany(cs.ns(|| "new h3"), &[current_hash_value[3].clone(), d])?;

        let h4 = e.compute(
            cs.ns(|| "deferred h4 computation"),
            &[current_hash_value[4].clone()],
        )?;

        let h5 = UInt32::addmany(cs.ns(|| "new h5"), &[current_hash_value[5].clone(), f])?;

        let h6 = UInt32::addmany(cs.ns(|| "new h6"), &[current_hash_value[6].clone(), g])?;

        let h7 = UInt32::addmany(cs.ns(|| "new h7"), &[current_hash_value[7].clone(), h])?;

        Ok(vec![h0, h1, h2, h3, h4, h5, h6, h7])
    }

    fn iv() -> Vec<UInt32> {
        IV.iter().map(|&v| UInt32::constant(v)).collect()
    }
}

impl<E: Engine> NativeFunction<E> for Sha256 {
    fn execute<CS: ConstraintSystem<E::Fr>>(
        &self,
        mut cs: CS,
        stack: &mut EvaluationStack<E>,
    ) -> Result<(), RuntimeError> {
        let mut bits = Vec::new();
        for i in 0..self.message_length {
            let bit = stack
                .pop()?
                .value()?
                .to_boolean(cs.ns(|| format!("bit {}", i)))?;

            bits.push(bit);
        }
        bits.reverse();

        let digest_bits =
            Self::run::<E::Fr, Namespace<'_, E::Fr, CS::Root>>(cs.ns(|| "sha256"), &bits)?;

        assert_eq!(digest_bits.len(), 256);

        for bit in digest_bits {
            let scalar = Scalar::from_boolean(cs.ns(|| "from_boolean"), bit)?;
            stack.push(scalar.into())?;
        }

        Ok(())
    }
}

trait Sha256Ext: Sized {
    fn maj<F, CS>(cs: CS, a: &Self, b: &Self, c: &Self) -> Result<Self, SynthesisError>
    where
        F: PrimeField,
        CS: ConstraintSystem<F>;

    fn ch<F, CS>(cs: CS, a: &Self, b: &Self, c: &Self) -> Result<Self, SynthesisError>
    where
        F: PrimeField,
        CS: ConstraintSystem<F>;
}

impl Sha256Ext for Boolean {
    fn maj<F, CS>(mut cs: CS, a: &Self, b: &Self, c: &Self) -> Result<Self, SynthesisError>
    where
        F: PrimeField,
        CS: ConstraintSystem<F>,
    {
        let maj_value = match (a.get_value(), b.get_value(), c.get_value()) {
            (Some(a), Some(b), Some(c)) => {
                // (a and b) xor (a and c) xor (b and c)
                Some((a & b) ^ (a & c) ^ (b & c))
            }
            _ => None,
        };

        match (a, b, c) {
            (&Boolean::Constant(_), &Boolean::Constant(_), &Boolean::Constant(_)) => {
                // They're all constants, so we can just compute the value.

                return Ok(Boolean::Constant(maj_value.expect("they're all constants")));
            }
            (&Boolean::Constant(false), b, c) => {
                // If a is false,
                // (a and b) xor (a and c) xor (b and c)
                // equals
                // (b and c)
                return Boolean::and(cs, b, c);
            }
            (a, &Boolean::Constant(false), c) => {
                // If b is false,
                // (a and b) xor (a and c) xor (b and c)
                // equals
                // (a and c)
                return Boolean::and(cs, a, c);
            }
            (a, b, &Boolean::Constant(false)) => {
                // If c is false,
                // (a and b) xor (a and c) xor (b and c)
                // equals
                // (a and b)
                return Boolean::and(cs, a, b);
            }
            (a, b, &Boolean::Constant(true)) => {
                // If c is true,
                // (a and b) xor (a and c) xor (b and c)
                // equals
                // (a and b) xor (a) xor (b)
                // equals
                // not ((not a) and (not b))
                return Ok(Boolean::and(cs, &a.not(), &b.not())?.not());
            }
            (a, &Boolean::Constant(true), c) => {
                // If b is true,
                // (a and b) xor (a and c) xor (b and c)
                // equals
                // (a) xor (a and c) xor (c)
                return Ok(Boolean::and(cs, &a.not(), &c.not())?.not());
            }
            (&Boolean::Constant(true), b, c) => {
                // If a is true,
                // (a and b) xor (a and c) xor (b and c)
                // equals
                // (b) xor (c) xor (b and c)
                return Ok(Boolean::and(cs, &b.not(), &c.not())?.not());
            }
            (&Boolean::Is(_), &Boolean::Is(_), &Boolean::Is(_))
            | (&Boolean::Is(_), &Boolean::Is(_), &Boolean::Not(_))
            | (&Boolean::Is(_), &Boolean::Not(_), &Boolean::Is(_))
            | (&Boolean::Is(_), &Boolean::Not(_), &Boolean::Not(_))
            | (&Boolean::Not(_), &Boolean::Is(_), &Boolean::Is(_))
            | (&Boolean::Not(_), &Boolean::Is(_), &Boolean::Not(_))
            | (&Boolean::Not(_), &Boolean::Not(_), &Boolean::Is(_))
            | (&Boolean::Not(_), &Boolean::Not(_), &Boolean::Not(_)) => {}
        }

        // ¬(¬a ∧ ¬b) ∧ ¬(¬a ∧ ¬c) ∧ ¬(¬b ∧ ¬c)
        // (1 - ((1 - a) * (1 - b))) * (1 - ((1 - a) * (1 - c))) * (1 - ((1 - b) * (1 - c)))
        // (a + b - ab) * (a + c - ac) * (b + c - bc)
        // -2abc + ab + ac + bc
        // a (-2bc + b + c) + bc
        //
        // (b) * (c) = (bc)
        // (2bc - b - c) * (a) = bc - maj

        let bc = Self::and(cs.ns(|| "b and c"), b, c)?;
        // FIXME get rid of this extra allocation
        let maj_bit = AllocatedBit::alloc(cs.ns(|| "maj"), || {
            maj_value.ok_or(SynthesisError::AssignmentMissing)
        })?;

        cs.enforce(
            || "maj computation",
            |_| {
                bc.lc(CS::one(), F::one()) + &bc.lc(CS::one(), F::one())
                    - &b.lc(CS::one(), F::one())
                    - &c.lc(CS::one(), F::one())
            },
            |_| a.lc(CS::one(), F::one()),
            |_| bc.lc(CS::one(), F::one()) - maj_bit.get_variable(),
        );

        Ok(maj_bit.into())
    }

    fn ch<F, CS>(mut cs: CS, a: &Self, b: &Self, c: &Self) -> Result<Self, SynthesisError>
    where
        F: PrimeField,
        CS: ConstraintSystem<F>,
    {
        let ch_value = match (a.get_value(), b.get_value(), c.get_value()) {
            (Some(a), Some(b), Some(c)) => {
                // (a and b) xor ((not a) and c)
                Some((a & b) ^ ((!a) & c))
            }
            _ => None,
        };

        match (a, b, c) {
            (&Self::Constant(_), &Self::Constant(_), &Self::Constant(_)) => {
                // They're all constants, so we can just compute the value.

                return Ok(Self::Constant(ch_value.expect("they're all constants")));
            }
            (&Self::Constant(false), _, c) => {
                // If a is false
                // (a and b) xor ((not a) and c)
                // equals
                // (false) xor (c)
                // equals
                // c
                return Ok(c.clone());
            }
            (a, &Self::Constant(false), c) => {
                // If b is false
                // (a and b) xor ((not a) and c)
                // equals
                // ((not a) and c)
                return Self::and(cs, &a.not(), &c);
            }
            (a, b, &Self::Constant(false)) => {
                // If c is false
                // (a and b) xor ((not a) and c)
                // equals
                // (a and b)
                return Self::and(cs, &a, &b);
            }
            (a, b, &Self::Constant(true)) => {
                // If c is true
                // (a and b) xor ((not a) and c)
                // equals
                // (a and b) xor (not a)
                // equals
                // not (a and (not b))
                return Ok(Self::and(cs, &a, &b.not())?.not());
            }
            (a, &Self::Constant(true), c) => {
                // If b is true
                // (a and b) xor ((not a) and c)
                // equals
                // a xor ((not a) and c)
                // equals
                // not ((not a) and (not c))
                return Ok(Self::and(cs, &a.not(), &c.not())?.not());
            }
            (&Self::Constant(true), _, _) => {
                // If a is true
                // (a and b) xor ((not a) and c)
                // equals
                // b xor ((not a) and c)
                // So we just continue!
            }
            (&Self::Is(_), &Self::Is(_), &Self::Is(_))
            | (&Self::Is(_), &Self::Is(_), &Self::Not(_))
            | (&Self::Is(_), &Self::Not(_), &Self::Is(_))
            | (&Self::Is(_), &Self::Not(_), &Self::Not(_))
            | (&Self::Not(_), &Self::Is(_), &Self::Is(_))
            | (&Self::Not(_), &Self::Is(_), &Self::Not(_))
            | (&Self::Not(_), &Self::Not(_), &Self::Is(_))
            | (&Self::Not(_), &Self::Not(_), &Self::Not(_)) => {}
        }

        // a(b - c) = ch - c

        // FIXME get rid of this extra allocation
        let ch_bit = AllocatedBit::alloc(cs.ns(|| "ch"), || {
            ch_value.ok_or(SynthesisError::AssignmentMissing)
        })?;
        cs.enforce(
            || "ch computation",
            |_| b.lc(CS::one(), F::one()) - &c.lc(CS::one(), F::one()),
            |_| a.lc(CS::one(), F::one()),
            |lc| lc + ch_bit.get_variable() - &c.lc(CS::one(), F::one()),
        );

        Ok(ch_bit.into())
    }
}

impl Sha256Ext for UInt32 {
    /// Compute the `maj` value (a and b) xor (a and c) xor (b and c)
    /// during SHA256.
    fn maj<F, CS>(cs: CS, a: &Self, b: &Self, c: &Self) -> Result<Self, SynthesisError>
    where
        F: PrimeField,
        CS: ConstraintSystem<F>,
    {
        Self::triop(
            cs,
            a,
            b,
            c,
            |a, b, c| (a & b) ^ (a & c) ^ (b & c),
            |cs, i, a, b, c| Boolean::maj(cs.ns(|| format!("maj {}", i)), a, b, c),
        )
    }

    /// Compute the `ch` value `(a and b) xor ((not a) and c)`
    /// during SHA256.
    fn ch<F, CS>(cs: CS, a: &Self, b: &Self, c: &Self) -> Result<Self, SynthesisError>
    where
        F: PrimeField,
        CS: ConstraintSystem<F>,
    {
        Self::triop(
            cs,
            a,
            b,
            c,
            |a, b, c| (a & b) ^ ((!a) & c),
            |cs, i, a, b, c| Boolean::ch(cs.ns(|| format!("ch {}", i)), a, b, c),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use algebra::bls12_381::Bls12;
    use r1cs_std::{bits::boolean::AllocatedBit, test_constraint_system::TestConstraintSystem};
    use rand::{Rng, SeedableRng, XorShiftRng};

    #[test]
    fn test_blank_hash() {
        let iv = Sha256::iv();

        let mut cs = TestConstraintSystem::<Bls12>::new();
        let mut input_bits: Vec<_> = (0..512).map(|_| Boolean::Constant(false)).collect();
        input_bits[0] = Boolean::Constant(true);
        let out = Sha256::compress(&mut cs, &input_bits, &iv).unwrap();
        let out_bits: Vec<_> = out.into_iter().flat_map(|e| e.into_bits_be()).collect();

        assert!(cs.is_satisfied());
        assert_eq!(cs.num_constraints(), 0);

        let expected = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");

        let mut out = out_bits.into_iter();
        for b in expected.into_iter() {
            for i in (0..8).rev() {
                let c = out.next().unwrap().get_value().unwrap();

                assert_eq!(c, (b >> i) & 1u8 == 1u8);
            }
        }
    }

    #[test]
    fn test_full_block() {
        let mut rng = XorShiftRng::from_seed([0x5dbe6259, 0x8d313d76, 0x3237db17, 0xe5bc0654]);

        let iv = Sha256::iv();

        let mut cs = TestConstraintSystem::<Bls12>::new();
        let input_bits: Vec<_> = (0..512)
            .map(|i| {
                Boolean::from(
                    AllocatedBit::alloc(
                        cs.namespace(|| format!("input bit {}", i)),
                        Some(rng.gen()),
                    )
                    .unwrap(),
                )
            })
            .collect();

        Sha256::compress(cs.namespace(|| "sha256"), &input_bits, &iv).unwrap();

        assert!(cs.is_satisfied());
        assert_eq!(cs.num_constraints() - 512, 25840);
    }

    #[test]
    fn test_against_vectors() {
        use sha2::{Digest, Sha256};

        let mut rng = XorShiftRng::from_seed([0x5dbe6259, 0x8d313d76, 0x3237db17, 0xe5bc0654]);

        for input_len in (0..32).chain((32..256).filter(|a| a % 8 == 0)) {
            let mut h = Sha256::new();
            let data: Vec<u8> = (0..input_len).map(|_| rng.gen()).collect();
            h.input(&data);
            let result = h.result();
            let hash_result = result.as_slice();

            let mut cs = TestConstraintSystem::<Bls12>::new();
            let mut input_bits = vec![];

            for (byte_i, input_byte) in data.into_iter().enumerate() {
                for bit_i in (0..8).rev() {
                    let cs = cs.namespace(|| format!("input bit {} {}", byte_i, bit_i));

                    input_bits.push(
                        AllocatedBit::alloc(cs, Some((input_byte >> bit_i) & 1u8 == 1u8))
                            .unwrap()
                            .into(),
                    );
                }
            }

            let r = Sha256::run(&mut cs, &input_bits).unwrap();

            assert!(cs.is_satisfied());

            let mut s = hash_result
                .as_ref()
                .iter()
                .flat_map(|&byte| (0..8).rev().map(move |i| (byte >> i) & 1u8 == 1u8));

            for b in r {
                match b {
                    Boolean::Is(b) => {
                        assert!(s.next().unwrap() == b.get_value().unwrap());
                    }
                    Boolean::Not(b) => {
                        assert!(s.next().unwrap() != b.get_value().unwrap());
                    }
                    Boolean::Constant(b) => {
                        assert!(input_len == 0);
                        assert!(s.next().unwrap() == b);
                    }
                }
            }
        }
    }
}

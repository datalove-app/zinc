use crate::gadgets::expression;
use crate::Engine;
use algebra::{BigInteger, BitIterator, Field, FpParameters, One, PrimeField, Zero};
use r1cs_core::{ConstraintSystem, LinearCombination, SynthesisError, Variable};
use r1cs_std::prelude::{AllocGadget, CondSelectGadget, ConditionalEqGadget, EqGadget};
use r1cs_std::{
    boolean::{AllocatedBit, Boolean},
    Assignment,
};
use std::ops::{Add, AddAssign, MulAssign, Sub, SubAssign};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AllocatedNum<E: Engine> {
    value: Option<E::Fr>,
    variable: Variable,
}

impl<E: Engine> AllocatedNum<E> {
    pub fn alloc<CS, F>(mut cs: CS, value: F) -> Result<Self, SynthesisError>
    where
        CS: ConstraintSystem<E::Fr>,
        F: FnOnce() -> Result<E::Fr, SynthesisError>,
    {
        let mut new_value = None;
        let var = cs.alloc(
            || "num",
            || {
                let tmp = value()?;
                new_value = Some(tmp);
                Ok(tmp)
            },
        )?;

        Ok(AllocatedNum {
            value: new_value,
            variable: var,
        })
    }

    // pub fn alloc_input<CS, F>(mut cs: CS, value: F) -> Result<Self, SynthesisError>
    // where
    //     CS: ConstraintSystem<E::Fr>,
    //     F: FnOnce() -> Result<E::Fr, SynthesisError>,
    // {
    //     let new_value = value();
    //     let variable = cs.alloc_input(|| "input num", || new_value)?;
    //
    //     Ok(AllocatedNum {
    //         value: new_value.ok(),
    //         variable,
    //     })
    // }

    pub fn inputize<CS>(&self, mut cs: CS) -> Result<(), SynthesisError>
    where
        CS: ConstraintSystem<E::Fr>,
    {
        let input = cs.alloc_input(|| "input variable", || Ok(self.value.get()?))?;

        cs.enforce(
            || "enforce input is correct",
            |zero| zero + input,
            |zero| zero + CS::one(),
            |zero| zero + self.variable,
        );

        Ok(())
    }

    pub fn one<CS>(_cs: CS) -> AllocatedNum<E>
    where
        CS: ConstraintSystem<E::Fr>,
    {
        AllocatedNum {
            value: Some(E::Fr::one()),
            variable: CS::one(),
        }
    }

    pub fn mul<CS>(&self, mut cs: CS, other: &Self) -> Result<Self, SynthesisError>
    where
        CS: ConstraintSystem<E::Fr>,
    {
        let mut value = None;

        let var = cs.alloc(
            || "product num",
            || {
                let mut tmp = self.value.get()?;
                tmp.mul_assign(&other.value.get()?);
                value = Some(tmp);
                Ok(tmp)
            },
        )?;

        // Constrain: a * b = ab
        cs.enforce(
            || "multiplication constraint",
            |zero| zero + self.variable,
            |zero| zero + other.variable,
            |zero| zero + var,
        );

        Ok(AllocatedNum {
            value: value,
            variable: var,
        })
    }

    pub fn square<CS>(&self, mut cs: CS) -> Result<Self, SynthesisError>
    where
        CS: ConstraintSystem<E::Fr>,
    {
        let mut value = None;

        let var = cs.alloc(
            || "squared num",
            || {
                let mut tmp = self.value.get()?;
                tmp.square_in_place();
                value = Some(tmp);
                Ok(tmp)
            },
        )?;

        // Constrain: a * a = aa
        cs.enforce(
            || "squaring constraint",
            |zero| zero + self.variable,
            |zero| zero + self.variable,
            |zero| zero + var,
        );

        Ok(AllocatedNum {
            value: value,
            variable: var,
        })
    }

    pub fn pow<CS>(&self, mut cs: CS, power: &E::Fr) -> Result<Self, SynthesisError>
    where
        CS: ConstraintSystem<E::Fr>,
    {
        let power_bits: Vec<bool> = BitIterator::new(power.into_repr()).collect();
        let mut temp = AllocatedNum::alloc(cs.ns(|| "one"), || Ok(E::Fr::one()))?;
        temp.assert_number(cs.ns(|| "assert_one"), &E::Fr::one())?;

        for (i, bit) in power_bits.iter().enumerate() {
            temp = temp.square(cs.ns(|| format!("square on step: {}", i)))?;
            if *bit {
                temp = temp.mul(cs.ns(|| format!("mul step: {}", i)), &self)?;
            }
        }

        Ok(temp)
    }

    /// Deconstructs this allocated number into its
    /// boolean representation in little-endian bit
    /// order, requiring that the representation
    /// strictly exists "in the field" (i.e., a
    /// congruency is not allowed.)
    pub fn to_bits_le_strict<CS>(&self, mut cs: CS) -> Result<Vec<Boolean>, SynthesisError>
    where
        CS: ConstraintSystem<E::Fr>,
    {
        pub fn kary_and<F, CS>(
            mut cs: CS,
            v: &[AllocatedBit],
        ) -> Result<AllocatedBit, SynthesisError>
        where
            F: PrimeField,
            CS: ConstraintSystem<F>,
        {
            assert!(v.len() > 0);

            // Let's keep this simple for now and just AND them all
            // manually
            let mut cur = None;

            for (i, v) in v.iter().enumerate() {
                if cur.is_none() {
                    cur = Some(v.clone());
                } else {
                    cur = Some(AllocatedBit::and(
                        cs.ns(|| format!("and {}", i)),
                        cur.as_ref().unwrap(),
                        v,
                    )?);
                }
            }

            Ok(cur.expect("v.len() > 0"))
        }

        // We want to ensure that the bit representation of a is
        // less than or equal to r - 1.
        let mut a = self.value.map(|e| BitIterator::new(e.into_repr()));
        let mut b = <<E as Engine>::Fr as PrimeField>::Params::MODULUS;
        b.sub_noborrow(&1.into());

        let mut result = vec![];

        // Runs of ones in r
        let mut last_run = None;
        let mut current_run = vec![];

        let mut found_one = false;
        let mut i = 0;
        for b in BitIterator::new(b) {
            let a_bit = a.as_mut().map(|e| e.next().unwrap());

            // Skip over unset bits at the beginning
            found_one |= b;
            if !found_one {
                // a_bit should also be false
                a_bit.map(|e| assert!(!e));
                continue;
            }

            if b {
                // This is part of a run of ones. Let's just
                // allocate the boolean with the expected value.
                let a_bit = AllocatedBit::alloc(cs.ns(|| format!("bit {}", i)), || {
                    a_bit.ok_or(SynthesisError::AssignmentMissing)
                })?;
                // ... and add it to the current run of ones.
                current_run.push(a_bit.clone());
                result.push(a_bit);
            } else {
                if current_run.len() > 0 {
                    // This is the start of a run of zeros, but we need
                    // to k-ary AND against `last_run` first.

                    if last_run.is_some() {
                        current_run.push(last_run.clone().unwrap());
                    }
                    last_run = Some(kary_and(
                        cs.ns(|| format!("run ending at {}", i)),
                        &current_run,
                    )?);
                    current_run.truncate(0);
                }

                // If `last_run` is true, `a` must be false, or it would
                // not be in the field.
                //
                // If `last_run` is false, `a` can be true or false.

                let a_bit = AllocatedBit::alloc_conditionally(
                    cs.ns(|| format!("bit {}", i)),
                    a_bit,
                    &last_run.as_ref().expect("char always starts with a one"),
                )?;
                result.push(a_bit);
            }

            i += 1;
        }

        // char is prime, so we'll always end on
        // a run of zeros.
        assert_eq!(current_run.len(), 0);

        // Now, we have `result` in big-endian order.
        // However, now we have to unpack self!

        let mut packed_lc = LinearCombination::zero();
        let mut coeff = E::Fr::one();

        for bit in result.iter().rev() {
            packed_lc = packed_lc + (coeff, bit.get_variable());
            coeff.double_in_place();
        }

        cs.enforce(
            || "unpacking constraint",
            |_| packed_lc,
            |zero| zero + CS::one(),
            |zero| zero + self.get_variable(),
        );

        // Convert into booleans, and reverse for little-endian bit order
        Ok(result.into_iter().map(|b| Boolean::from(b)).rev().collect())
    }

    /// Convert the allocated number into its little-endian representation.
    /// Note that this does not strongly enforce that the commitment is
    /// "in the field."
    pub fn to_bits_le<CS>(&self, mut cs: CS) -> Result<Vec<Boolean>, SynthesisError>
    where
        CS: ConstraintSystem<E::Fr>,
    {
        let bits = expression::field_into_allocated_bits_le::<CS, E::Fr>(&mut cs, self.value)?;

        let mut packed_lc = LinearCombination::zero();
        let mut coeff = E::Fr::one();

        for bit in bits.iter() {
            packed_lc = packed_lc + (coeff, bit.get_variable());
            coeff.double_in_place();
        }

        cs.enforce(
            || "unpacking constraint",
            |_| packed_lc,
            |zero| zero + CS::one(),
            |zero| zero + self.get_variable(),
        );

        Ok(bits.into_iter().map(|b| Boolean::from(b)).collect())
    }

    /// Return fixed amount of bits of the allocated number.
    /// Should be used when there is a priori knowledge of bit length of the number
    pub fn to_bits_le_fixed<CS>(
        &self,
        mut cs: CS,
        bit_length: usize,
    ) -> Result<Vec<Boolean>, SynthesisError>
    where
        CS: ConstraintSystem<E::Fr>,
    {
        let bits = expression::field_into_allocated_bits_le_fixed::<CS, E::Fr>(
            &mut cs, self.value, bit_length,
        )?;

        let mut packed_lc = LinearCombination::zero();
        let mut coeff = E::Fr::one();

        for bit in bits.iter() {
            packed_lc = packed_lc + (coeff, bit.get_variable());
            coeff.double_in_place();
        }

        cs.enforce(
            || "unpacking constraint",
            |_| packed_lc,
            |zero| zero + CS::one(),
            |zero| zero + self.get_variable(),
        );

        Ok(bits.into_iter().map(|b| Boolean::from(b)).collect())
    }

    /// Return allocated number given its bit representation
    pub fn pack_bits_to_element<CS>(mut cs: CS, bits: &[Boolean]) -> Result<Self, SynthesisError>
    where
        CS: ConstraintSystem<E::Fr>,
    {
        let mut data_from_lc = Num::<E>::zero();
        let mut coeff = E::Fr::one();
        for bit in bits {
            data_from_lc = data_from_lc.add_bool_with_coeff(CS::one(), &bit, coeff);
            coeff.double_in_place();
        }

        let data_packed = AllocatedNum::alloc(cs.ns(|| "allocate packed number"), || {
            Ok(data_from_lc.get_value().get()?)
        })?;

        cs.enforce(
            || "pack bits to number",
            |zero| zero + data_packed.get_variable(),
            |zero| zero + CS::one(),
            |_| data_from_lc.lc(E::Fr::one()),
        );

        Ok(data_packed)
    }

    /*
        pub fn invert<CS>(&self, mut cs: CS) -> Result<AllocatedNum<F>, SynthesisError>
        where
            CS: ConstraintSystem<F>,
        {
            let mut newval = None;
            let newnum = AllocatedNum::alloc(cs.ns(|| "inverse"), || {
                let inv = self
                    .value
                    .ok_or(SynthesisError::AssignmentMissing)?
                    .invert();
                if bool::from(inv.is_some()) {
                    let tmp = inv.unwrap();
                    newval = Some(tmp);
                    Ok(tmp)
                } else {
                    Err(SynthesisError::Unsatisfiable)
                }
            })?;

            let (a, b, c) = cs.multiply(
                || "invert",
                || {
                    Ok((
                        newval.ok_or(SynthesisError::AssignmentMissing)?,
                        self.value.ok_or(SynthesisError::AssignmentMissing)?,
                        F::one(),
                    ))
                },
            )?;

            cs.enforce_zero(LinearCombination::from(a) - newnum.get_variable());
            cs.enforce_zero(LinearCombination::from(b) - self.get_variable());
            cs.enforce_zero(LinearCombination::from(c) - CS::ONE);

            Ok(newnum)
        }

        pub fn sqrt<CS>(&self, mut cs: CS) -> Result<AllocatedNum<F>, SynthesisError>
        where
            CS: ConstraintSystem<F>,
        {
            let mut newval = None;
            let newnum = AllocatedNum::alloc(cs.ns(|| "sqrt"), || {
                let sqrt = self.value.ok_or(SynthesisError::AssignmentMissing)?.sqrt();
                if bool::from(sqrt.is_some()) {
                    let tmp = sqrt.unwrap();
                    newval = Some(tmp);
                    Ok(tmp)
                } else {
                    Err(SynthesisError::Unsatisfiable)
                }
            })?;

            let (a, b, c) = cs.multiply(
                || "square root check",
                || {
                    Ok((
                        newval.ok_or(SynthesisError::AssignmentMissing)?,
                        newval.ok_or(SynthesisError::AssignmentMissing)?,
                        self.value.ok_or(SynthesisError::AssignmentMissing)?,
                    ))
                },
            )?;

            cs.enforce_zero(LinearCombination::from(a) - newnum.get_variable());
            cs.enforce_zero(LinearCombination::from(b) - newnum.get_variable());
            cs.enforce_zero(LinearCombination::from(c) - self.get_variable());

            Ok(newnum)
        }
    */

    /*
    pub fn rescue_alpha<CS>(mut cs: CS, base: &Combination<F>) -> Result<Self, SynthesisError>
    where
        F: Field,
        CS: ConstraintSystem<F>,
    {
        let base_value = base.get_value();
        let result_value = base_value.and_then(|num| Some(num.pow(&[F::RESCUE_ALPHA, 0, 0, 0])));

        // base^5 --> Constrain base^5 = result
        assert_eq!(F::RESCUE_ALPHA, 5);
        let (base_var, result_var) =
            constrain_pow_five(cs.ns(|| "constrain base^5"), base_value)?;

        let base_lc = base.lc(&mut cs);
        cs.enforce_zero(base_lc - base_var);

        Ok(AllocatedNum {
            value: result_value,
            var: result_var,
        })
    }

    pub fn rescue_invalpha<CS>(mut cs: CS, base: &Combination<F>) -> Result<Self, SynthesisError>
    where
        F: Field,
        CS: ConstraintSystem<F>,
    {
        let base_value = base.get_value();
        let result_value = base_value.and_then(|num| Some(num.pow(&F::RESCUE_INVALPHA)));

        // base^(1/5) --> Constrain result^5 = base
        assert_eq!(F::RESCUE_ALPHA, 5);
        let (result_var, base_var) =
            constrain_pow_five(cs.ns(|| "constrain result^5"), result_value)?;

        let base_lc = base.lc(&mut cs);
        cs.enforce_zero(base_lc - base_var);

        Ok(AllocatedNum {
            value: result_value,
            var: result_var,
        })
    }
     */

    pub fn assert_nonzero<CS>(&self, mut cs: CS) -> Result<(), SynthesisError>
    where
        CS: ConstraintSystem<E::Fr>,
    {
        let inv = cs.alloc(
            || "ephemeral inverse",
            || {
                let tmp = self.value.get()?;
                if tmp.is_zero() {
                    Err(SynthesisError::DivisionByZero)
                } else {
                    Ok(tmp.inverse().unwrap())
                }
            },
        )?;

        // Constrain a * inv = 1, which is only valid
        // iff a has a multiplicative inverse, untrue
        // for zero.
        cs.enforce(
            || "nonzero assertion constraint",
            |zero| zero + self.variable,
            |zero| zero + inv,
            |zero| zero + CS::one(),
        );

        Ok(())
    }

    pub fn assert_zero<CS>(&self, mut cs: CS) -> Result<(), SynthesisError>
    where
        CS: ConstraintSystem<E::Fr>,
    {
        cs.enforce(
            || "zero assertion constraint",
            |zero| zero + self.variable,
            |zero| zero + CS::one(),
            |zero| zero,
        );

        Ok(())
    }

    pub fn assert_number<CS>(&self, mut cs: CS, number: &E::Fr) -> Result<(), SynthesisError>
    where
        CS: ConstraintSystem<E::Fr>,
    {
        cs.enforce(
            || "number assertion constraint",
            |zero| zero + self.variable - (number.clone(), CS::one()),
            |zero| zero + CS::one(),
            |zero| zero,
        );

        Ok(())
    }

    /// Takes two allocated numbers (a, b) and returns
    /// (b, a) if the condition is true, and (a, b)
    /// otherwise.
    pub fn conditionally_reverse<CS>(
        mut cs: CS,
        a: &Self,
        b: &Self,
        condition: &Boolean,
    ) -> Result<(Self, Self), SynthesisError>
    where
        CS: ConstraintSystem<E::Fr>,
    {
        let c = Self::alloc(cs.ns(|| "conditional reversal result 1"), || {
            if condition.get_value().get()? {
                Ok(b.value.get()?)
            } else {
                Ok(a.value.get()?)
            }
        })?;

        cs.enforce(
            || "first conditional reversal",
            |zero| zero + a.variable - b.variable,
            |_| condition.lc(CS::one(), E::Fr::one()),
            |zero| zero + a.variable - c.variable,
        );

        let d = Self::alloc(cs.ns(|| "conditional reversal result 2"), || {
            if condition.get_value().get()? {
                Ok(a.value.get()?)
            } else {
                Ok(b.value.get()?)
            }
        })?;

        cs.enforce(
            || "second conditional reversal",
            |zero| zero + b.variable - a.variable,
            |_| condition.lc(CS::one(), E::Fr::one()),
            |zero| zero + b.variable - d.variable,
        );

        Ok((c, d))
    }

    /// Takes two allocated numbers (a, b) and returns
    /// allocated boolean variable with value `true`
    /// if the `a` and `b` are equal, `false` otherwise.
    pub fn equals<CS>(mut cs: CS, a: &Self, b: &Self) -> Result<AllocatedBit, SynthesisError>
    where
        CS: ConstraintSystem<E::Fr>,
    {
        // Allocate and constrain `r`: result boolean bit.
        // It equals `true` if `a` equals `b`, `false` otherwise

        let r_value = match (a.value, b.value) {
            (Some(a), Some(b)) => Some(a == b),
            _ => None,
        };

        let r = AllocatedBit::alloc(cs.ns(|| "r"), || {
            r_value.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // Let `delta = a - b`

        let delta_value = match (a.value, b.value) {
            (Some(a), Some(b)) => {
                // return (a - b)
                let mut a = a;
                a.sub_assign(&b);
                Some(a)
            }
            _ => None,
        };

        let delta_inv_value = delta_value.as_ref().map(|delta_value| {
            let tmp = delta_value.clone();
            if tmp.is_zero() {
                E::Fr::one() // we can return any number here, it doesn't matter
            } else {
                tmp.inverse().unwrap()
            }
        });

        let delta_inv = Self::alloc(cs.ns(|| "delta_inv"), || {
            delta_inv_value.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // Allocate `t = delta * delta_inv`
        // If `delta` is non-zero (a != b), `t` will equal 1
        // If `delta` is zero (a == b), `t` cannot equal 1

        let t_value = match (delta_value, delta_inv_value) {
            (Some(a), Some(b)) => {
                let mut t = a.clone();
                t.mul_assign(&b);
                Some(t)
            }
            _ => None,
        };

        let t = Self::alloc(cs.ns(|| "t"), || {
            t_value.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // Constrain allocation:
        // t = (a - b) * delta_inv
        cs.enforce(
            || "t = (a - b) * delta_inv",
            |zero| zero + a.variable - b.variable,
            |zero| zero + delta_inv.variable,
            |zero| zero + t.variable,
        );

        // Constrain:
        // (a - b) * (t - 1) == 0
        // This enforces that correct `delta_inv` was provided,
        // and thus `t` is 1 if `(a - b)` is non zero (a != b )
        cs.enforce(
            || "(a - b) * (t - 1) == 0",
            |zero| zero + a.variable - b.variable,
            |zero| zero + t.variable - CS::one(),
            |zero| zero,
        );

        // Constrain:
        // (a - b) * r == 0
        // This enforces that `r` is zero if `(a - b)` is non-zero (a != b)
        cs.enforce(
            || "(a - b) * r == 0",
            |zero| zero + a.variable - b.variable,
            |zero| zero + r.get_variable(),
            |zero| zero,
        );

        // Constrain:
        // (t - 1) * (r - 1) == 0
        // This enforces that `r` is one if `t` is not one (a == b)
        cs.enforce(
            || "(t - 1) * (r - 1) == 0",
            |zero| zero + t.get_variable() - CS::one(),
            |zero| zero + r.get_variable() - CS::one(),
            |zero| zero,
        );

        Ok(r)
    }

    /// Returns `a == b ? x : y`
    pub fn select_ifeq<CS>(
        mut cs: CS,
        a: &Self,
        b: &Self,
        x: &Self,
        y: &Self,
    ) -> Result<Self, SynthesisError>
    where
        CS: ConstraintSystem<E::Fr>,
    {
        let eq = Self::equals(cs.ns(|| "eq"), a, b)?;
        Self::conditionally_select(cs.ns(|| "select"), &Boolean::from(eq), x, y)
    }

    /// Limits number of bits. The easiest example when required
    /// is to add or subtract two "small" (with bit length smaller
    /// than one of the field) numbers and check for overflow
    pub fn limit_number_of_bits<CS>(
        &self,
        mut cs: CS,
        number_of_bits: usize,
    ) -> Result<(), SynthesisError>
    where
        CS: ConstraintSystem<E::Fr>,
    {
        // do the bit decomposition and check that higher bits are all zeros

        let mut bits = self.to_bits_le(cs.ns(|| "unpack to limit number of bits"))?;

        bits.drain(0..number_of_bits);

        // repack

        let mut top_bits_lc = Num::<E>::zero();
        let mut coeff = E::Fr::one();
        for bit in bits.into_iter() {
            top_bits_lc = top_bits_lc.add_bool_with_coeff(CS::one(), &bit, coeff);
            coeff.double_in_place();
        }

        // enforce packing and zeroness
        cs.enforce(
            || "repack top bits",
            |zero| zero,
            |zero| zero + CS::one(),
            |_| top_bits_lc.lc(E::Fr::one()),
        );

        Ok(())
    }

    // pub fn from_raw_unchecked(value: Option<E::Fr>, variable: Variable) -> Self {
    //     AllocatedNum { value, variable }
    // }

    pub fn lc(&self) -> LinearCombination<E::Fr> {
        LinearCombination::from(self.variable)
    }

    pub fn get_value(&self) -> Option<E::Fr> {
        self.value
    }

    pub fn get_variable(&self) -> Variable {
        self.variable
    }
}

// impl<E: Field> ConditionalEqGadget<F> for AllocatedNum<F> {
//     fn conditional_enforce_equal<CS: ConstraintSystem<F>>(
//         &self,
//         mut cs: CS,
//         other: &Self,
//         condition: &Boolean,
//     ) -> Result<(), SynthesisError> {
//         // TODO: how early to short-circuit if condition is false?
//
//         let difference = self.sub(cs.ns(|| "difference"), other)?;
//         let one = CS::one();
//         let one_const = F::one();
//         cs.enforce(
//             || "conditional_equals",
//             |lc| lc + &difference.variable,
//             |lc| lc + &condition.lc(one, one_const),
//             |lc| lc,
//         );
//         Ok(())
//     }
//
//     fn cost() -> usize {
//         todo!()
//     }
// }

// impl<F: Field> EqGadget<F> for AllocatedNum<F> {}

impl<E: Engine> CondSelectGadget<E::Fr> for AllocatedNum<E> {
    fn conditionally_select<CS: ConstraintSystem<E::Fr>>(
        mut cs: CS,
        cond: &Boolean,
        true_value: &Self,
        false_value: &Self,
    ) -> Result<Self, SynthesisError> {
        let c = Self::alloc(cs.ns(|| "conditional select result"), || {
            if cond.get_value().get()? {
                Ok(true_value.value.get()?)
            } else {
                Ok(false_value.value.get()?)
            }
        })?;

        // a * condition + b*(1-condition) = c ->
        // a * condition - b*condition = c - b

        cs.enforce(
            || "conditional select constraint",
            |zero| zero + true_value.variable - false_value.variable,
            |_| cond.lc(CS::one(), E::Fr::one()),
            |zero| zero + c.variable - false_value.variable,
        );

        Ok(c)
    }

    fn cost() -> usize {
        todo!()
    }
}

// /// Constrain (x)^5 = (x^5), and return variables for x and (x^5).
// ///
// /// We can do so with three multiplication constraints and five linear constraints:
// ///
// /// a * b = c
// /// a := x
// /// b = a
// /// c := x^2
// ///
// /// d * e = f
// /// d = c
// /// e = c
// /// f := x^4
// ///
// /// g * h = i
// /// g = f
// /// h = x
// /// i := x^5
// fn constrain_pow_five<F, CS>(
//     mut cs: CS,
//     x: Option<F>,
// ) -> Result<(Variable, Variable), SynthesisError>
// where
//     F: Field,
//     CS: ConstraintSystem<F>,
// {
//     let x2 = x.and_then(|x| Some(x.square()));
//     let x4 = x2.and_then(|x2| Some(x2.square()));
//     let x5 = x4.and_then(|x4| x.and_then(|x| Some(x4 * x)));
//
//     let (base_var, b_var, c_var) = cs.multiply(
//         || "x^2",
//         || {
//             let x = x.ok_or(SynthesisError::AssignmentMissing)?;
//             let x2 = x2.ok_or(SynthesisError::AssignmentMissing)?;
//
//             Ok((x, x, x2))
//         },
//     )?;
//     cs.enforce_zero(LinearCombination::from(base_var) - b_var);
//
//     let (d_var, e_var, f_var) = cs.multiply(
//         || "x^4",
//         || {
//             let x2 = x2.ok_or(SynthesisError::AssignmentMissing)?;
//             let x4 = x4.ok_or(SynthesisError::AssignmentMissing)?;
//
//             Ok((x2, x2, x4))
//         },
//     )?;
//     cs.enforce_zero(LinearCombination::from(c_var) - d_var);
//     cs.enforce_zero(LinearCombination::from(c_var) - e_var);
//
//     let (g_var, h_var, result_var) = cs.multiply(
//         || "x^5",
//         || {
//             let x = x.ok_or(SynthesisError::AssignmentMissing)?;
//             let x4 = x4.ok_or(SynthesisError::AssignmentMissing)?;
//             let x5 = x5.ok_or(SynthesisError::AssignmentMissing)?;
//
//             Ok((x4, x, x5))
//         },
//     )?;
//     cs.enforce_zero(LinearCombination::from(f_var) - g_var);
//     cs.enforce_zero(LinearCombination::from(base_var) - h_var);
//
//     Ok((base_var, result_var))
// }

impl<E: Engine> From<AllocatedBit> for AllocatedNum<E> {
    fn from(bit: AllocatedBit) -> AllocatedNum<E> {
        AllocatedNum {
            variable: bit.get_variable(),
            value: bit
                .get_value()
                .map(|v| if v { E::Fr::one() } else { E::Fr::zero() }),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Num<E: Engine> {
    value: Option<E::Fr>,
    lc: LinearCombination<E::Fr>,
}

impl<E: Engine> From<AllocatedNum<E>> for Num<E> {
    fn from(num: AllocatedNum<E>) -> Num<E> {
        Num {
            value: num.value,
            lc: LinearCombination::<E::Fr>::zero() + num.variable,
        }
    }
}

impl<E: Engine> Num<E> {
    pub fn zero() -> Self {
        Num {
            value: Some(E::Fr::zero()),
            lc: LinearCombination::zero(),
        }
    }

    pub fn get_value(&self) -> Option<E::Fr> {
        self.value
    }

    pub fn lc(&self, coeff: E::Fr) -> LinearCombination<E::Fr> {
        LinearCombination::zero() + (coeff, &self.lc)
    }

    pub fn add_number_with_coeff(self, variable: &AllocatedNum<E>, coeff: E::Fr) -> Self {
        let newval = match (self.value, variable.get_value()) {
            (Some(mut curval), Some(val)) => {
                let mut tmp = val;
                tmp.mul_assign(&coeff);

                curval.add_assign(&tmp);

                Some(curval)
            }
            _ => None,
        };

        Num {
            value: newval,
            lc: self.lc + (coeff, variable.get_variable()),
        }
    }

    pub fn add_bool_with_coeff(self, one: Variable, bit: &Boolean, coeff: E::Fr) -> Self {
        let newval = match (self.value, bit.get_value()) {
            (Some(mut curval), Some(bval)) => {
                if bval {
                    curval.add_assign(&coeff);
                }

                Some(curval)
            }
            _ => None,
        };

        Num {
            value: newval,
            lc: self.lc + &bit.lc(one, coeff),
        }
    }
}

impl<E: Engine> Add<&Num<E>> for Num<E> {
    type Output = Num<E>;

    fn add(self, other: &Num<E>) -> Num<E> {
        let newval = match (self.value, other.value) {
            (Some(mut curval), Some(val)) => {
                curval.add_assign(&val);
                Some(curval)
            }
            _ => None,
        };

        Num {
            value: newval,
            lc: self.lc + &other.lc,
        }
    }
}

impl<E: Engine> Sub<&Num<E>> for Num<E> {
    type Output = Num<E>;

    fn sub(self, other: &Num<E>) -> Num<E> {
        let newval = match (self.value, other.value) {
            (Some(mut curval), Some(val)) => {
                curval.sub_assign(&val);
                Some(curval)
            }
            _ => None,
        };

        Num {
            value: newval,
            lc: self.lc - &other.lc,
        }
    }
}

// #[derive(Clone, Copy, Debug)]
// pub enum Num<E: Field> {
//     Constant(Coeff<E>),
//     Allocated(Coeff<F>, AllocatedNum<F>),
// }
//
// impl<F: Field> Neg for Num<F> {
//     type Output = Self;
//
//     fn neg(self) -> Self {
//         match self {
//             Num::Constant(coeff) => Num::Constant(-coeff),
//             Num::Allocated(coeff, var) => Num::Allocated(-coeff, var),
//         }
//     }
// }
//
// impl<F: Field> From<AllocatedNum<F>> for Num<F> {
//     fn from(num: AllocatedNum<F>) -> Self {
//         Num::Allocated(Coeff::One, num)
//     }
// }

// impl<F: Field> From<(Coeff<F>, AllocatedNum<F>)> for Num<F> {
//     fn from(num: (Coeff<F>, AllocatedNum<F>)) -> Self {
//         Num::Allocated(num.0, num.1)
//     }
// }
//
// impl<F: Field> From<(Coeff<F>, Num<F>)> for Num<F> {
//     fn from(num: (Coeff<F>, Num<F>)) -> Self {
//         match num.1 {
//             Num::Constant(coeff) => Num::Constant(num.0 * coeff),
//             Num::Allocated(coeff, n) => Num::Allocated(num.0 * coeff, n),
//         }
//     }
// }

// impl<F: Field> Num<F> {
//     pub fn scale(self, val: F) -> Self {
//         match self {
//             Num::Constant(coeff) => Num::Constant(coeff * val),
//             Num::Allocated(coeff, var) => Num::Allocated(coeff * val, var),
//         }
//     }
//
//     pub fn constant(val: F) -> Self {
//         Num::Constant(Coeff::from(val))
//     }
//
//     pub fn is_constant(&self) -> bool {
//         match *self {
//             Num::Constant(_) => true,
//             _ => false,
//         }
//     }
//
//     pub fn value(&self) -> Option<F> {
//         match *self {
//             Num::Constant(v) => Some(v.value()),
//             Num::Allocated(coeff, var) => var.value.map(|v| (coeff * v).value()),
//         }
//     }
//
//     pub fn lc<CS: ConstraintSystem<F>>(&self, mut _cs: CS) -> LinearCombination<F> {
//         LinearCombination::zero()
//             + match self {
//                 Num::Constant(v) => (*v, CS::ONE),
//                 Num::Allocated(coeff, num) => (*coeff, num.var),
//             }
//     }
// }
//
// #[derive(Clone, Debug)]
// pub struct Combination<F: Field> {
//     value: Option<F>,
//     terms: Vec<Num<F>>,
// }
//
// impl<F: Field> From<AllocatedNum<F>> for Combination<F> {
//     fn from(num: AllocatedNum<F>) -> Self {
//         Combination {
//             value: num.value,
//             terms: vec![num.into()],
//         }
//     }
// }
//
// impl<F: Field> From<Num<F>> for Combination<F> {
//     fn from(num: Num<F>) -> Self {
//         Combination {
//             value: num.value(),
//             terms: vec![num],
//         }
//     }
// }
//
// impl<F: Field> Add<Combination<F>> for Combination<F> {
//     type Output = Combination<F>;
//
//     fn add(mut self, other: Combination<F>) -> Combination<F> {
//         let new_value = self
//             .value
//             .and_then(|a| other.value.and_then(|b| Some(a + b)));
//
//         self.terms.extend(other.terms);
//
//         Combination {
//             value: new_value,
//             terms: self.terms,
//         }
//     }
// }
//
// impl<F: Field> Add<AllocatedNum<F>> for Combination<F> {
//     type Output = Combination<F>;
//
//     fn add(mut self, other: AllocatedNum<F>) -> Combination<F> {
//         self += other;
//         self
//     }
// }
//
// impl<'a, F: Field> AddAssign<AllocatedNum<F>> for Combination<F> {
//     fn add_assign(&mut self, other: AllocatedNum<F>) {
//         self.value = self
//             .value
//             .and_then(|a| other.value.and_then(|b| Some(a + b)));
//
//         self.terms.push(other.into());
//     }
// }
//
// impl<F: Field> Sub<AllocatedNum<F>> for Combination<F> {
//     type Output = Combination<F>;
//
//     fn sub(mut self, other: AllocatedNum<F>) -> Combination<F> {
//         let new_value = self
//             .value
//             .and_then(|a| other.value.and_then(|b| Some(a - b)));
//
//         self.terms.push(-Num::from(other));
//
//         Combination {
//             value: new_value,
//             terms: self.terms,
//         }
//     }
// }
//
// impl<F: Field> Add<Num<F>> for Combination<F> {
//     type Output = Combination<F>;
//
//     fn add(mut self, other: Num<F>) -> Combination<F> {
//         self += other;
//         self
//     }
// }
//
// impl<'a, F: Field> AddAssign<Num<F>> for Combination<F> {
//     fn add_assign(&mut self, other: Num<F>) {
//         self.value = self
//             .value
//             .and_then(|a| other.value().and_then(|b| Some(a + b)));
//
//         self.terms.push(other);
//     }
// }

// impl<F: Field> Add<(Coeff<F>, AllocatedNum<F>)> for Combination<F> {
//     type Output = Combination<F>;
//
//     fn add(mut self, other: (Coeff<F>, AllocatedNum<F>)) -> Combination<F> {
//         let new_value = self
//             .value
//             .and_then(|a| other.1.value.and_then(|b| Some(a + (other.0.value() * b))));
//
//         self.terms.push(other.into());
//
//         Combination {
//             value: new_value,
//             terms: self.terms,
//         }
//     }
// }
//
// impl<F: Field> Add<(Coeff<F>, Num<F>)> for Combination<F> {
//     type Output = Combination<F>;
//
//     fn add(mut self, other: (Coeff<F>, Num<F>)) -> Combination<F> {
//         let new_value = self.value.and_then(|a| {
//             other
//                 .1
//                 .value()
//                 .and_then(|b| Some(a + (other.0.value() * b)))
//         });
//
//         self.terms.push(other.into());
//
//         Combination {
//             value: new_value,
//             terms: self.terms,
//         }
//     }
// }

// impl<F: Field> Combination<F> {
//     pub fn zero() -> Self {
//         Combination {
//             value: Some(F::zero()),
//             terms: vec![],
//         }
//     }
//
//     pub fn scale(self, by: F) -> Self {
//         let value = self.value.map(|v| v * by);
//         let terms = self.terms.into_iter().map(|t| t.scale(by)).collect();
//
//         Combination { value, terms }
//     }
//
//     pub fn get_value(&self) -> Option<F> {
//         self.value
//     }
//
//     pub fn lc<CS: ConstraintSystem<F>>(&self, mut cs: CS) -> LinearCombination<F> {
//         let mut acc = LinearCombination::zero();
//
//         for term in &self.terms {
//             acc = acc + &term.lc(&mut cs);
//         }
//
//         acc
//     }
//
//     pub fn evaluate<CS>(&self, mut cs: CS) -> Result<Num<F>, SynthesisError>
//     where
//         CS: ConstraintSystem<F>,
//     {
//         let any_allocated = self.terms.iter().any(|n| !n.is_constant());
//
//         if any_allocated {
//             let out = AllocatedNum::alloc(cs.ns(|| "combination"), || {
//                 self.value.ok_or(SynthesisError::AssignmentMissing)
//             })?;
//             let lc = self.lc(&mut cs);
//             cs.enforce_zero(out.lc() - &lc);
//             Ok(out.into())
//         } else {
//             // We can just return a constant
//             let base_value = self.value.ok_or(SynthesisError::AssignmentMissing)?;
//             Ok(Num::constant(base_value))
//         }
//     }
//
//     pub fn mul<CS: ConstraintSystem<F>>(
//         &self,
//         mut cs: CS,
//         other: &Combination<F>,
//     ) -> Result<AllocatedNum<F>, SynthesisError> {
//         let mut value = None;
//         let (l, r, o) = cs.multiply(
//             || "mul",
//             || {
//                 let l = self.value.ok_or(SynthesisError::AssignmentMissing)?;
//                 let r = other.value.ok_or(SynthesisError::AssignmentMissing)?;
//                 let o = l * &r;
//                 value = Some(o);
//
//                 Ok((l, r, o))
//             },
//         )?;
//
//         let lc = self.lc(&mut cs);
//         cs.enforce_zero(lc - l);
//         let lc = other.lc(&mut cs);
//         cs.enforce_zero(lc - r);
//
//         Ok(AllocatedNum { value, var: o })
//     }
//
//     pub fn square<CS: ConstraintSystem<F>>(
//         &self,
//         mut cs: CS,
//     ) -> Result<AllocatedNum<F>, SynthesisError> {
//         let mut value = None;
//         let (l, r, o) = cs.multiply(
//             || "square",
//             || {
//                 let l = self.value.ok_or(SynthesisError::AssignmentMissing)?;
//                 let c = l.square();
//                 value = Some(c);
//
//                 Ok((l, l, c))
//             },
//         )?;
//
//         let lc = self.lc(&mut cs);
//         cs.enforce_zero(lc.clone() - l);
//         cs.enforce_zero(lc - r);
//
//         Ok(AllocatedNum { value, var: o })
//     }
//
//     pub fn rescue_alpha<CS>(&self, cs: CS) -> Result<Num<F>, SynthesisError>
//     where
//         CS: ConstraintSystem<F>,
//     {
//         let any_allocated = self.terms.iter().any(|n| !n.is_constant());
//
//         if any_allocated {
//             AllocatedNum::rescue_alpha(cs, self).map(|n| n.into())
//         } else {
//             // We can just return a constant
//             let base_value = self.value.ok_or(SynthesisError::AssignmentMissing)?;
//             Ok(Num::constant(base_value.pow(&[F::RESCUE_ALPHA, 0, 0, 0])))
//         }
//     }
//
//     pub fn rescue_invalpha<CS>(&self, cs: CS) -> Result<Num<F>, SynthesisError>
//     where
//         CS: ConstraintSystem<F>,
//     {
//         let any_allocated = self.terms.iter().any(|n| !n.is_constant());
//
//         if any_allocated {
//             AllocatedNum::rescue_invalpha(cs, self).map(|n| n.into())
//         } else {
//             // We can just return a constant
//             let base_value = self.value.ok_or(SynthesisError::AssignmentMissing)?;
//             Ok(Num::constant(base_value.pow(&F::RESCUE_INVALPHA)))
//         }
//     }
// }

#[cfg(test)]
mod test {
    use super::AllocatedNum;
    use r1cs_core::{ConstraintSynthesizer, ConstraintSystem, SynthesisError};

    // use crate::{
    //     circuits::{Circuit, ConstraintSystem, SynthesisError},
    //     dev::is_satisfied,
    //     fields::Fp,
    //     Basic,
    // };

    #[test]
    fn test_allocated_num() {
        #[derive(Default)]
        struct TestCircuit;
        impl ConstraintSynthesizer<Fp> for TestCircuit {
            fn generate_constraints<CS: ConstraintSystem<Fp>>(
                &self,
                cs: &mut CS,
            ) -> Result<(), SynthesisError> {
                let _ = AllocatedNum::alloc(cs, || Ok(Fp::one()))?;
                Ok(())
            }
        }

        assert_eq!(
            is_satisfied::<_, _, Basic>(&TestCircuit::default(), &[]),
            Ok(true)
        );
    }

    #[test]
    fn test_num_alloc_and_square() {
        #[derive(Default)]
        struct TestCircuit;
        impl ConstraintSynthesizer<Fp> for TestCircuit {
            fn generate_constraints<CS: ConstraintSystem<Fp>>(
                &self,
                cs: &mut CS,
            ) -> Result<(), SynthesisError> {
                let (n, n2) = AllocatedNum::alloc_and_square(cs, || Ok(Fp::from(3)))?;

                assert!(n.value.unwrap() == Fp::from(3));
                assert!(n2.value.unwrap() == Fp::from(9));

                Ok(())
            }
        }

        assert_eq!(
            is_satisfied::<_, _, Basic>(&TestCircuit::default(), &[]),
            Ok(true)
        );
    }

    #[test]
    fn test_num_multiplication() {
        #[derive(Default)]
        struct TestCircuit;
        impl ConstraintSynthesizer<Fp> for TestCircuit {
            fn generate_constraints<CS: ConstraintSystem<Fp>>(
                &self,
                cs: &mut CS,
            ) -> Result<(), SynthesisError> {
                let n = AllocatedNum::alloc(cs.ns(|| "a"), || Ok(Fp::from(12)))?;
                let n2 = AllocatedNum::alloc(cs.ns(|| "b"), || Ok(Fp::from(10)))?;
                let n3 = n.mul(cs, &n2)?;
                assert!(n3.value.unwrap() == Fp::from(120));

                Ok(())
            }
        }

        assert_eq!(
            is_satisfied::<_, _, Basic>(&TestCircuit::default(), &[]),
            Ok(true)
        );
    }
}

mod scalar_type;
pub use scalar_type::*;

use crate::gadgets::{utils, AllocatedNum, Expression};
use crate::{Result, RuntimeError};
use algebra::{Field, PrimeField};
use num_bigint::{BigInt, ToBigInt};
use num_traits::ToPrimitive;
use r1cs_core::{ConstraintSystem, LinearCombination, SynthesisError, Variable};
use r1cs_std::{
    alloc::AllocGadget,
    boolean::{AllocatedBit, Boolean},
    Assignment,
};
use std::fmt;

/// Scalar is a primitive value that can be stored on the stack and operated by VM's instructions.
#[derive(Debug, Clone)]
pub struct Scalar<F: Field> {
    variant: ScalarVariant<F>,
    scalar_type: ScalarType,
}

#[derive(Debug, Clone)]
pub enum ScalarVariant<F: Field> {
    Constant(ScalarConstant<F>),
    Variable(ScalarVariable<F>),
}

impl<F: Field> From<ScalarConstant<F>> for ScalarVariant<F> {
    fn from(constant: ScalarConstant<F>) -> Self {
        Self::Constant(constant)
    }
}

impl<F: Field> From<ScalarVariable<F>> for ScalarVariant<F> {
    fn from(variable: ScalarVariable<F>) -> Self {
        Self::Variable(variable)
    }
}

#[derive(Debug, Clone)]
pub struct ScalarConstant<F: Field> {
    pub value: F,
}

#[derive(Debug, Clone)]
pub struct ScalarVariable<F: Field> {
    value: Option<F>,
    variable: Variable,
}

impl<F: Field> Scalar<F> {
    pub fn new_constant_int(value: usize, scalar_type: ScalarType) -> Self {
        let value_string = value.to_string();
        let fr = F::from_str(&value_string).expect("failed to convert u64 into Fr");
        Self::new_constant_fr(fr, scalar_type)
    }

    pub fn new_constant_bool(value: bool) -> Self {
        let fr = if value { F::one() } else { F::zero() };
        Self::new_constant_fr(fr, ScalarType::Boolean)
    }

    pub fn new_constant_fr(value: F, scalar_type: ScalarType) -> Self {
        Self {
            variant: ScalarConstant { value }.into(),
            scalar_type,
        }
    }

    pub fn new_constant_bigint(value: &BigInt, scalar_type: ScalarType) -> Result<Self> {
        let fr = utils::bigint_to_fr::<F>(value).ok_or(RuntimeError::ValueOverflow {
            value: value.clone(),
            scalar_type,
        })?;
        Ok(Self::new_constant_fr(fr, scalar_type))
    }

    pub fn new_unchecked_variable(
        value: Option<F>,
        variable: Variable,
        scalar_type: ScalarType,
    ) -> Self {
        Self {
            variant: ScalarVariable { value, variable }.into(),
            scalar_type,
        }
    }

    pub fn to_expression<CS: ConstraintSystem<F>>(&self) -> Expression<F> {
        Expression::new(self.get_value(), self.lc::<CS>())
    }

    pub fn to_boolean<CS: ConstraintSystem<F>>(&self, mut cs: CS) -> Result<Boolean> {
        self.scalar_type.assert_type(ScalarType::Boolean)?;

        match &self.variant {
            ScalarVariant::Constant(constant) => Ok(Boolean::constant(!constant.value.is_zero())),
            ScalarVariant::Variable(variable) => {
                let bit = AllocatedBit::alloc(
                    cs.ns(|| "allocate bit"),
                    || variable.value
                        .ok_or(SynthesisError::AssignmentMissing)
                        .map(|value| !value.is_zero()),
                )?;

                cs.enforce(
                    || "bit equality",
                    |zero| zero + bit.get_variable(),
                    |zero| zero + CS::one(),
                    |zero| zero + variable.variable,
                );

                Ok(bit.into())
            }
        }
    }

    pub fn get_type(&self) -> ScalarType {
        self.scalar_type
    }

    pub fn get_value(&self) -> Option<F> {
        match &self.variant {
            ScalarVariant::Constant(constant) => Some(constant.value),
            ScalarVariant::Variable(variable) => variable.value,
        }
    }

    pub fn get_variant(&self) -> &ScalarVariant<F> {
        &self.variant
    }

    pub fn grab_value(&self) -> std::result::Result<F, SynthesisError> {
        match self.get_value() {
            Some(v) => Ok(v.clone()),
            None => Err(SynthesisError::AssignmentMissing),
        }
    }

    pub fn get_constant(&self) -> Result<F> {
        match &self.variant {
            ScalarVariant::Constant(constant) => Ok(constant.value),
            _ => Err(RuntimeError::ExpectedConstant),
        }
    }

    pub fn get_constant_usize(&self) -> Result<usize> {
        let fr = self.get_constant()?;
        let bigint = utils::fr_to_bigint(&fr, false);
        bigint
            .to_usize()
            .ok_or_else(|| RuntimeError::ExpectedUsize(bigint))
    }

    pub fn as_field(&self) -> Self {
        Self {
            variant: self.variant.clone(),
            scalar_type: ScalarType::Field,
        }
    }

    pub fn is_signed(&self) -> bool {
        self.scalar_type.is_signed()
    }

    pub fn is_constant(&self) -> bool {
        match self.variant {
            ScalarVariant::Constant(_) => true,
            ScalarVariant::Variable(_) => false,
        }
    }

    pub fn lc<CS: ConstraintSystem<F>>(&self) -> LinearCombination<F> {
        match &self.variant {
            ScalarVariant::Constant(constant) => {
                LinearCombination::zero() + (constant.value, CS::one())
            }
            ScalarVariant::Variable(variable) => LinearCombination::zero() + variable.variable,
        }
    }

    pub fn get_bits_le<CS: ConstraintSystem<F>>(&self, mut cs: CS) -> Result<Vec<Self>>
    where
        F: PrimeField,
    {
        let num = self.to_expression::<CS>();
        let bits = match self.scalar_type {
            ScalarType::Field => num.into_bits_le_strict(cs.ns(|| "into_bits_le_strict")),
            scalar_type => num.into_bits_le_fixed(
                cs.ns(|| "into_bits_le_fixed"),
                scalar_type.bit_length::<F>(),
            ),
        }?;

        bits.into_iter()
            .enumerate()
            .map(|(i, bit)| Self::from_boolean(cs.ns(|| format!("bit {}", i)), bit))
            .collect()
    }

    pub fn with_type_unchecked(&self, scalar_type: ScalarType) -> Self {
        Self {
            variant: self.variant.clone(),
            scalar_type,
        }
    }

    pub fn from_boolean<CS: ConstraintSystem<F>>(mut cs: CS, boolean: Boolean) -> Result<Self> {
        match boolean {
            Boolean::Is(bit) => Ok(Self::new_unchecked_variable(
                bit.get_value_field::<F>(),
                bit.get_variable(),
                ScalarType::Boolean,
            )),
            Boolean::Not(bit) => {
                let expr = Expression::constant::<CS>(F::one()) - Expression::from(&bit);
                let num = expr.into_number(cs.ns(|| "into_number"))?;
                let scalar = Self::from(num);
                Ok(scalar.with_type_unchecked(ScalarType::Boolean))
            }
            Boolean::Constant(_) => Ok(Self::new_constant_fr(
                boolean.get_value_field::<F>().unwrap(),
                ScalarType::Boolean,
            )),
        }
    }

    pub fn as_constant_unchecked(&self) -> Result<Self> {
        Ok(Self::new_constant_fr(self.grab_value()?, self.get_type()))
    }
}

//impl<F: Field> fmt::Debug for Scalar<F> {
//    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//        let value_str = self
//            .value
//            .map(|f| utils::fr_to_bigint(&f, self.is_signed()).to_string())
//            .unwrap_or_else(|| "none".into());
//
//        write!(
//            f,
//            "Scalar {{ value: {}, type: {} }}",
//            value_str, self.scalar_type
//        )
//    }
//}

impl<F: Field> fmt::Display for Scalar<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value_str = self
            .get_value()
            .map(|f| utils::fr_to_bigint(&f, self.is_signed()).to_string())
            .unwrap_or_else(|| "none".into());

        let det = if self.is_constant() { "det" } else { "witness" };
        write!(f, "{} as {} ({})", value_str, self.scalar_type, det)
    }
}

impl<F: PrimeField> ToBigInt for Scalar<F> {
    fn to_bigint(&self) -> Option<BigInt> {
        self.get_value()
            .map(|fr| utils::fr_to_bigint(&fr, self.is_signed()))
    }
}

impl<F: Field> From<&AllocatedNum<F>> for Scalar<F> {
    fn from(num: &AllocatedNum<F>) -> Self {
        Self {
            variant: ScalarVariable {
                value: num.get_value(),
                variable: num.get_variable(),
            }
            .into(),
            scalar_type: ScalarType::Field,
        }
    }
}

impl<F: Field> From<AllocatedNum<F>> for Scalar<F> {
    fn from(num: AllocatedNum<F>) -> Self {
        Self {
            variant: ScalarVariable {
                value: num.get_value(),
                variable: num.get_variable(),
            }
            .into(),
            scalar_type: ScalarType::Field,
        }
    }
}

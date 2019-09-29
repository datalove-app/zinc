//!
//! The interpreter value.
//!

mod boolean;
mod error;
mod integer;

pub use self::boolean::Boolean;
pub use self::boolean::Error as BooleanError;
pub use self::error::Error;
pub use self::integer::Error as IntegerError;
pub use self::integer::Integer;

use std::fmt;

use num_bigint::BigInt;
use num_traits::Zero;

use r1cs::Bn256;
use r1cs::ConstraintSystem;

use crate::lexical::BooleanLiteral;
use crate::lexical::IntegerLiteral;
use crate::syntax::Type;
use crate::syntax::TypeVariant;

#[derive(Clone, PartialEq)]
pub enum Value {
    Void,
    Boolean(Boolean),
    Integer(Integer),
    Array(Vec<Value>),
}

impl Value {
    pub fn new_boolean<CS: ConstraintSystem<Bn256>>(
        mut system: CS,
        boolean: BooleanLiteral,
    ) -> Result<Self, Error> {
        Boolean::new_from_literal(system.namespace(|| "value_new_boolean"), boolean)
            .map(Self::Boolean)
            .map_err(Error::Boolean)
    }

    pub fn new_integer<CS: ConstraintSystem<Bn256>>(
        mut system: CS,
        integer: IntegerLiteral,
    ) -> Result<Self, Error> {
        Integer::new_from_literal(system.namespace(|| "value_new_integer"), integer)
            .map(Self::Integer)
            .map_err(Error::Integer)
    }

    pub fn new_input<CS: ConstraintSystem<Bn256>>(
        mut system: CS,
        r#type: Type,
    ) -> Result<Self, Error> {
        match r#type.variant {
            TypeVariant::Void => Ok(Self::Void),
            TypeVariant::Boolean => {
                Boolean::new_from_bool(system.namespace(|| "value_new_input"), false)
                    .map(Self::Boolean)
                    .map_err(Error::Boolean)
            }
            TypeVariant::Int { bitlength } => Integer::new_from_bigint(
                system.namespace(|| "value_new_input"),
                BigInt::zero(),
                true,
                bitlength,
            )
            .map(Self::Integer)
            .map_err(Error::Integer),
            TypeVariant::Uint { bitlength } => Integer::new_from_bigint(
                system.namespace(|| "value_new_input"),
                BigInt::zero(),
                false,
                bitlength,
            )
            .map(Self::Integer)
            .map_err(Error::Integer),
            TypeVariant::Field => Integer::new_from_bigint(
                system.namespace(|| "value_new_input"),
                BigInt::zero(),
                false,
                crate::SIZE_FIELD,
            )
            .map(Self::Integer)
            .map_err(Error::Integer),
            TypeVariant::Array { .. } => unimplemented!(),
        }
    }

    pub fn new_witness<CS: ConstraintSystem<Bn256>>(
        mut system: CS,
        r#type: Type,
    ) -> Result<Self, Error> {
        match r#type.variant {
            TypeVariant::Void => Ok(Self::Void),
            TypeVariant::Boolean => {
                Boolean::new_from_bool(system.namespace(|| "value_new_witness"), false)
                    .map(Self::Boolean)
                    .map_err(Error::Boolean)
            }
            TypeVariant::Int { bitlength } => Integer::new_from_bigint(
                system.namespace(|| "value_new_witness"),
                BigInt::zero(),
                true,
                bitlength,
            )
            .map(Self::Integer)
            .map_err(Error::Integer),
            TypeVariant::Uint { bitlength } => Integer::new_from_bigint(
                system.namespace(|| "value_new_witness"),
                BigInt::zero(),
                false,
                bitlength,
            )
            .map(Self::Integer)
            .map_err(Error::Integer),
            TypeVariant::Field => Integer::new_from_bigint(
                system.namespace(|| "value_new_witness"),
                BigInt::zero(),
                false,
                crate::SIZE_FIELD,
            )
            .map(Self::Integer)
            .map_err(Error::Integer),
            TypeVariant::Array { .. } => unimplemented!(),
        }
    }

    pub fn has_the_same_type_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Void, Self::Void) => true,
            (Self::Boolean(..), Self::Boolean(..)) => true,
            (Self::Integer(integer_1), Self::Integer(integer_2)) => {
                integer_1.has_the_same_type_as(integer_2)
            }
            _ => false,
        }
    }

    pub fn is_of_type(&self, r#type: &TypeVariant) -> bool {
        match (self, r#type) {
            (Self::Void, TypeVariant::Void) => true,
            (Self::Boolean(..), TypeVariant::Boolean) => true,
            (Self::Integer(integer), TypeVariant::Uint { bitlength }) => {
                integer.bitlength == *bitlength && !integer.is_signed
            }
            (Self::Integer(integer), TypeVariant::Int { bitlength }) => {
                integer.bitlength == *bitlength && integer.is_signed
            }
            (Self::Integer(integer), TypeVariant::Field) => {
                integer.bitlength == crate::SIZE_FIELD && !integer.is_signed
            }
            (Self::Array(array), TypeVariant::Array { r#type, size }) => {
                if let Some(element) = array.get(0) {
                    element.is_of_type(&r#type.variant) && array.len() == *size
                } else {
                    true
                }
            }
            _ => false,
        }
    }

    pub fn or<CS: ConstraintSystem<Bn256>>(
        self,
        mut system: CS,
        other: Self,
    ) -> Result<Self, Error> {
        let value_1 = match self {
            Self::Boolean(value) => value,
            value => return Err(Error::ExpectedBoolean("or", value)),
        };

        let value_2 = match other {
            Self::Boolean(value) => value,
            value => return Err(Error::ExpectedBoolean("or", value)),
        };

        value_1
            .or(system.namespace(|| "value_or"), value_2)
            .map(Self::Boolean)
            .map_err(Error::Boolean)
    }

    pub fn xor<CS: ConstraintSystem<Bn256>>(
        self,
        mut system: CS,
        other: Self,
    ) -> Result<Self, Error> {
        let value_1 = match self {
            Self::Boolean(value) => value,
            value => return Err(Error::ExpectedBoolean("xor", value)),
        };

        let value_2 = match other {
            Self::Boolean(value) => value,
            value => return Err(Error::ExpectedBoolean("xor", value)),
        };

        value_1
            .xor(system.namespace(|| "value_xor"), value_2)
            .map(Self::Boolean)
            .map_err(Error::Boolean)
    }

    pub fn and<CS: ConstraintSystem<Bn256>>(
        self,
        mut system: CS,
        other: Self,
    ) -> Result<Self, Error> {
        let value_1 = match self {
            Self::Boolean(value) => value,
            value => return Err(Error::ExpectedBoolean("and", value)),
        };

        let value_2 = match other {
            Self::Boolean(value) => value,
            value => return Err(Error::ExpectedBoolean("and", value)),
        };

        value_1
            .and(system.namespace(|| "value_and"), value_2)
            .map(Self::Boolean)
            .map_err(Error::Boolean)
    }

    pub fn equals<CS: ConstraintSystem<Bn256>>(
        &self,
        mut system: CS,
        other: &Self,
    ) -> Result<Self, Error> {
        match (self, other) {
            (Self::Void, Self::Void) => {
                Boolean::new_from_bool(system.namespace(|| "value_equals"), true)
                    .map(Self::Boolean)
                    .map_err(Error::Boolean)
            }
            (Self::Boolean(value_1), Self::Boolean(value_2)) => value_1
                .equals(system.namespace(|| "value_equals"), value_2)
                .map(Self::Boolean)
                .map_err(Error::Boolean),
            (Self::Boolean(..), value_2) => {
                Err(Error::ExpectedBoolean("equals", value_2.to_owned()))
            }
            (Self::Integer(value_1), Self::Integer(value_2)) => value_1
                .equals(system.namespace(|| "value_equals"), value_2)
                .map(Self::Boolean)
                .map_err(Error::Integer),
            (Self::Integer(..), value_2) => {
                Err(Error::ExpectedInteger("equals", value_2.to_owned()))
            }
            (value_1, value_2) => Err(Error::OperandTypesMismatch(
                value_1.to_owned(),
                value_2.to_owned(),
            )),
        }
    }

    pub fn not_equals<CS: ConstraintSystem<Bn256>>(
        &self,
        mut system: CS,
        other: &Self,
    ) -> Result<Self, Error> {
        match (self, other) {
            (Self::Void, Self::Void) => {
                Boolean::new_from_bool(system.namespace(|| "value_not_equals"), false)
                    .map(Self::Boolean)
                    .map_err(Error::Boolean)
            }
            (Self::Boolean(value_1), Self::Boolean(value_2)) => value_1
                .not_equals(system.namespace(|| "value_not_equals"), value_2)
                .map(Self::Boolean)
                .map_err(Error::Boolean),
            (Self::Boolean(..), value_2) => {
                Err(Error::ExpectedBoolean("not_equals", value_2.to_owned()))
            }
            (Self::Integer(value_1), Self::Integer(value_2)) => value_1
                .not_equals(system.namespace(|| "value_not_equals"), value_2)
                .map(Self::Boolean)
                .map_err(Error::Integer),
            (Self::Integer(..), value_2) => {
                Err(Error::ExpectedInteger("not_equals", value_2.to_owned()))
            }
            (value_1, value_2) => Err(Error::OperandTypesMismatch(
                value_1.to_owned(),
                value_2.to_owned(),
            )),
        }
    }

    pub fn greater_equals<CS: ConstraintSystem<Bn256>>(
        self,
        mut system: CS,
        other: &Self,
    ) -> Result<Self, Error> {
        let value_1 = match self {
            Self::Integer(value) => value,
            value => return Err(Error::ExpectedInteger("greater_equals", value)),
        };

        let value_2 = match other {
            Self::Integer(value) => value,
            value => return Err(Error::ExpectedInteger("greater_equals", value.to_owned())),
        };

        value_1
            .greater_equals(system.namespace(|| "value_greater_equals"), &value_2)
            .map(Self::Boolean)
            .map_err(Error::Integer)
    }

    pub fn lesser_equals<CS: ConstraintSystem<Bn256>>(
        self,
        mut system: CS,
        other: &Self,
    ) -> Result<Self, Error> {
        let value_1 = match self {
            Self::Integer(value) => value,
            value => return Err(Error::ExpectedInteger("lesser_equals", value)),
        };

        let value_2 = match other {
            Self::Integer(value) => value,
            value => return Err(Error::ExpectedInteger("lesser_equals", value.to_owned())),
        };

        value_1
            .lesser_equals(system.namespace(|| "value_lesser_equals"), &value_2)
            .map(Self::Boolean)
            .map_err(Error::Integer)
    }

    pub fn greater<CS: ConstraintSystem<Bn256>>(
        self,
        mut system: CS,
        other: &Self,
    ) -> Result<Self, Error> {
        let value_1 = match self {
            Self::Integer(value) => value,
            value => return Err(Error::ExpectedInteger("greater", value)),
        };

        let value_2 = match other {
            Self::Integer(value) => value,
            value => return Err(Error::ExpectedInteger("greater", value.to_owned())),
        };

        value_1
            .greater(system.namespace(|| "value_greater"), &value_2)
            .map(Self::Boolean)
            .map_err(Error::Integer)
    }

    pub fn lesser<CS: ConstraintSystem<Bn256>>(
        self,
        mut system: CS,
        other: &Self,
    ) -> Result<Self, Error> {
        let value_1 = match self {
            Self::Integer(value) => value,
            value => return Err(Error::ExpectedInteger("lesser", value)),
        };

        let value_2 = match other {
            Self::Integer(value) => value,
            value => return Err(Error::ExpectedInteger("lesser", value.to_owned())),
        };

        value_1
            .lesser(system.namespace(|| "value_lesser"), &value_2)
            .map(Self::Boolean)
            .map_err(Error::Integer)
    }

    pub fn add<CS: ConstraintSystem<Bn256>>(
        self,
        mut system: CS,
        other: Self,
    ) -> Result<Self, Error> {
        let value_1 = match self {
            Self::Integer(value) => value,
            value => return Err(Error::ExpectedInteger("add", value)),
        };

        let value_2 = match other {
            Self::Integer(value) => value,
            value => return Err(Error::ExpectedInteger("add", value)),
        };

        value_1
            .add(system.namespace(|| "value_add"), value_2)
            .map(Self::Integer)
            .map_err(Error::Integer)
    }

    pub fn subtract<CS: ConstraintSystem<Bn256>>(
        self,
        mut system: CS,
        other: Self,
    ) -> Result<Self, Error> {
        let value_1 = match self {
            Self::Integer(value) => value,
            value => return Err(Error::ExpectedInteger("subtract", value)),
        };

        let value_2 = match other {
            Self::Integer(value) => value,
            value => return Err(Error::ExpectedInteger("subtract", value)),
        };

        value_1
            .subtract(system.namespace(|| "value_subtract"), value_2)
            .map(Self::Integer)
            .map_err(Error::Integer)
    }

    pub fn multiply<CS: ConstraintSystem<Bn256>>(
        self,
        mut system: CS,
        other: Self,
    ) -> Result<Self, Error> {
        let value_1 = match self {
            Self::Integer(value) => value,
            value => return Err(Error::ExpectedInteger("multiply", value)),
        };

        let value_2 = match other {
            Self::Integer(value) => value,
            value => return Err(Error::ExpectedInteger("multiply", value)),
        };

        value_1
            .multiply(system.namespace(|| "value_multiply"), value_2)
            .map(Self::Integer)
            .map_err(Error::Integer)
    }

    pub fn divide<CS: ConstraintSystem<Bn256>>(
        self,
        _system: CS,
        _other: Self,
    ) -> Result<Self, Error> {
        //        let value_1 = match self {
        //            Self::Integer(value) => value,
        //            value => return Err(Error::ExpectedIntegerValue("divide", value)),
        //        };
        //
        //        let value_2 = match other {
        //            Self::Integer(value) => value,
        //            value => return Err(Error::ExpectedIntegerValue("divide", value)),
        //        };
        //
        //        value_1
        //            .divide(system.namespace(|| "value_divide"), value_2)
        //            .map(Self::Integer)
        //            .map_err(Error::Integer)

        unimplemented!();
    }

    pub fn modulo<CS: ConstraintSystem<Bn256>>(
        self,
        _system: CS,
        _other: Self,
    ) -> Result<Self, Error> {
        //        let value_1 = match self {
        //            Self::Integer(value) => value,
        //            value => return Err(Error::ExpectedIntegerValue("modulo", value)),
        //        };
        //
        //        let value_2 = match other {
        //            Self::Integer(value) => value,
        //            value => return Err(Error::ExpectedIntegerValue("modulo", value)),
        //        };
        //
        //        value_1
        //            .modulo(system.namespace(|| "value_modulo"), value_2)
        //            .map(Self::Integer)
        //            .map_err(Error::Integer)

        unimplemented!();
    }

    pub fn negate<CS: ConstraintSystem<Bn256>>(self, mut system: CS) -> Result<Self, Error> {
        let value = match self {
            Self::Integer(value) => value,
            value => return Err(Error::ExpectedInteger("negate", value)),
        };

        value
            .negate(system.namespace(|| "value_negate"))
            .map(Self::Integer)
            .map_err(Error::Integer)
    }

    pub fn not<CS: ConstraintSystem<Bn256>>(self, mut system: CS) -> Result<Self, Error> {
        let value = match self {
            Self::Boolean(value) => value,
            value => return Err(Error::ExpectedBoolean("not", value)),
        };

        value
            .not(system.namespace(|| "value_not"))
            .map(Self::Boolean)
            .map_err(Error::Boolean)
    }

    pub fn cast<CS: ConstraintSystem<Bn256>>(
        self,
        mut system: CS,
        r#type: Type,
    ) -> Result<Self, Error> {
        let value = match self {
            Self::Integer(value) => value,
            value => return Err(Error::ExpectedInteger("cast", value)),
        };

        value
            .cast(system.namespace(|| "value_cast"), r#type.variant)
            .map(Self::Integer)
            .map_err(Error::Integer)
    }

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Void => write!(f, "()"),
            Self::Boolean(boolean) => write!(f, "{}", boolean),
            Self::Integer(integer) => write!(f, "{}", integer),
            Self::Array(array) => write!(f, "{:?}", array),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt(f)
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt(f)
    }
}

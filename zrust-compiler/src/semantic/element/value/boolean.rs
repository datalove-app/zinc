//!
//! The semantic analyzer element boolean value.
//!

use std::fmt;

use num_bigint::BigInt;
use num_traits::One;
use num_traits::Zero;

use zrust_bytecode::Push;

use crate::lexical::BooleanLiteral;
use crate::syntax::TypeVariant;

#[derive(Default, Clone, PartialEq)]
pub struct Boolean {
    pub value: Option<bool>,
}

impl Boolean {
    pub fn type_variant(&self) -> TypeVariant {
        TypeVariant::new_boolean()
    }

    pub fn to_push(&self) -> Push {
        let value = self.value.expect("Must contain a value");
        Push::new(
            if value { BigInt::one() } else { BigInt::zero() },
            false,
            crate::BITLENGTH_BYTE,
        )
    }

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

impl From<bool> for Boolean {
    fn from(value: bool) -> Self {
        Self { value: Some(value) }
    }
}

impl From<BooleanLiteral> for Boolean {
    fn from(value: BooleanLiteral) -> Self {
        Self {
            value: Some(value.into()),
        }
    }
}

impl fmt::Display for Boolean {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt(f)
    }
}

impl fmt::Debug for Boolean {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt(f)
    }
}

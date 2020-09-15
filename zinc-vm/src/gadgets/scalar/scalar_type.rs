use crate::RuntimeError;
use algebra::{FpParameters, PrimeField};
pub use zinc_bytecode::scalar::*;

pub trait ScalarTypeExpectation: Sized {
    fn expect_same(left: Self, right: Self) -> Result<Self, RuntimeError>;
    fn assert_type(&self, expected: Self) -> Result<(), RuntimeError>;
    fn bit_length<F: PrimeField>(&self) -> usize;
}

impl ScalarTypeExpectation for ScalarType {
    fn expect_same(left: Self, right: Self) -> Result<Self, RuntimeError> {
        if left == right {
            Ok(left)
        } else {
            Err(RuntimeError::TypeError {
                expected: left.to_string(),
                actual: right.to_string(),
            })
        }
    }

    fn assert_type(&self, expected: Self) -> Result<(), RuntimeError> {
        if self == &expected {
            Ok(())
        } else {
            Err(RuntimeError::TypeError {
                expected: expected.to_string(),
                actual: self.to_string(),
            })
        }
    }

    fn bit_length<F: PrimeField>(&self) -> usize {
        match self {
            ScalarType::Field => F::Params::MODULUS_BITS as usize,
            ScalarType::Boolean => 1,
            ScalarType::Integer(int_type) => int_type.bitlength,
        }
    }
}

pub mod array;
pub mod bits;
pub mod crypto;
pub mod ff;

use crate::core::EvaluationStack;
use crate::{Engine, Result};
use r1cs_core::ConstraintSystem;

pub trait NativeFunction<E: Engine> {
    fn execute<CS: ConstraintSystem<E::Fr>>(
        &self,
        cs: CS,
        stack: &mut EvaluationStack<E>,
    ) -> Result;
}

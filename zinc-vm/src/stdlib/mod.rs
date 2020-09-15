pub mod array;
pub mod bits;
pub mod crypto;
pub mod ff;

use crate::core::EvaluationStack;
use crate::Result;
use algebra::Field;
use r1cs_core::ConstraintSystem;

pub trait NativeFunction<F: Field> {
    fn execute<CS: ConstraintSystem<F>>(&self, cs: CS, stack: &mut EvaluationStack<F>) -> Result;
}

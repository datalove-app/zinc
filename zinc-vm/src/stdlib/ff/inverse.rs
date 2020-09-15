use crate::core::EvaluationStack;
use crate::stdlib::NativeFunction;
use crate::{gadgets, Result};
use algebra::Field;
use r1cs_core::ConstraintSystem;

pub struct Inverse;

impl<F: Field> NativeFunction<F> for Inverse {
    fn execute<CS>(&self, cs: CS, stack: &mut EvaluationStack<F>) -> Result
    where
        CS: ConstraintSystem<F>,
    {
        let scalar = stack.pop()?.value()?;
        let inverse = gadgets::arithmetic::inverse(cs, &scalar)?;
        stack.push(inverse.into())
    }
}

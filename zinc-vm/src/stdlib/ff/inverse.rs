use crate::core::EvaluationStack;
use crate::stdlib::NativeFunction;
use crate::{gadgets, Engine, Result};
use r1cs_core::ConstraintSystem;

pub struct Inverse;

impl<E: Engine> NativeFunction<E> for Inverse {
    fn execute<CS>(&self, cs: CS, stack: &mut EvaluationStack<E>) -> Result
    where
        CS: ConstraintSystem<E::Fr>,
    {
        let scalar = stack.pop()?.value()?;
        let inverse = gadgets::arithmetic::inverse(cs, &scalar)?;
        stack.push(inverse.into())
    }
}

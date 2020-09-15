use crate::core::EvaluationStack;
use crate::stdlib::NativeFunction;
use crate::{MalformedBytecode, Result};
use algebra::Field;
use r1cs_core::ConstraintSystem;

pub struct Truncate {
    array_length: usize,
}

impl Truncate {
    pub fn new(inputs_count: usize) -> Result<Self> {
        inputs_count
            .checked_sub(1)
            .map(|array_length| Self { array_length })
            .ok_or_else(|| {
                MalformedBytecode::InvalidArguments(
                    "array::truncate expects at least 2 arguments".into(),
                )
                .into()
            })
    }
}

impl<F: Field> NativeFunction<F> for Truncate {
    fn execute<CS: ConstraintSystem<F>>(&self, _cs: CS, stack: &mut EvaluationStack<F>) -> Result {
        let new_length = stack.pop()?.value()?.get_constant_usize()?;

        if new_length > self.array_length {
            return Err(MalformedBytecode::InvalidArguments(
                "array::truncate: new length can't be smaller".into(),
            )
            .into());
        }

        for _ in 0..(self.array_length - new_length) {
            stack.pop()?;
        }

        Ok(())
    }
}

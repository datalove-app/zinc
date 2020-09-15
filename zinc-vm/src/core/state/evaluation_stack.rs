use crate::core::Cell;
use crate::errors::MalformedBytecode;
use crate::gadgets;
use crate::gadgets::Scalar;
use crate::RuntimeError;
use algebra::Field;
use r1cs_core::ConstraintSystem;
use std::fmt;

#[derive(Debug)]
pub struct EvaluationStack<F: Field> {
    stack: Vec<Vec<Cell<F>>>,
}

impl<F: Field> EvaluationStack<F> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            stack: vec![vec![]],
        }
    }

    pub fn push(&mut self, value: Cell<F>) -> Result<(), RuntimeError> {
        self.stack
            .last_mut()
            .ok_or_else(|| {
                RuntimeError::InternalError("Evaluation stack root frame missing".into())
            })?
            .push(value);
        Ok(())
    }

    pub fn pop(&mut self) -> Result<Cell<F>, RuntimeError> {
        self.stack
            .last_mut()
            .ok_or_else(|| {
                RuntimeError::InternalError("Evaluation stack root frame missing".into())
            })?
            .pop()
            .ok_or_else(|| MalformedBytecode::StackUnderflow.into())
    }

    pub fn fork(&mut self) {
        self.stack.push(vec![]);
    }

    pub fn merge<CS>(&mut self, mut cs: CS, condition: &Scalar<F>) -> Result<(), RuntimeError>
    where
        CS: ConstraintSystem<F>,
    {
        let else_case = self.stack.pop().ok_or_else(|| {
            RuntimeError::InternalError("Evaluation stack root frame missing".into())
        })?;
        let then_case = self.stack.pop().ok_or_else(|| {
            RuntimeError::InternalError("Evaluation stack root frame missing".into())
        })?;

        if then_case.len() != else_case.len() {
            return Err(MalformedBytecode::BranchStacksDoNotMatch.into());
        }

        for (i, (t, e)) in then_case.into_iter().zip(else_case.into_iter()).enumerate() {
            match (t, e) {
                (Cell::Value(tv), Cell::Value(ev)) => {
                    let merged = gadgets::conditional_select(
                        cs.ns(|| format!("merge {}", i)),
                        condition,
                        &tv,
                        &ev,
                    )?;
                    self.push(Cell::Value(merged))?;
                }
            }
        }

        Ok(())
    }

    pub fn revert(&mut self) -> Result<(), RuntimeError> {
        self.stack.pop().ok_or(MalformedBytecode::StackUnderflow)?;
        Ok(())
    }
}

impl<F: Field> fmt::Display for EvaluationStack<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Evaluation Stack:")?;

        for frame in self.stack.iter().rev() {
            for cell in frame.iter().rev() {
                let Cell::Value(value) = cell;
                writeln!(f, "\t{}", value)?;
            }
        }

        Ok(())
    }
}

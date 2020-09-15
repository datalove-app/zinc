use crate::gadgets::Scalar;
use crate::RuntimeError;
use algebra::Field;

#[derive(Debug, Clone)]
pub enum Cell<F: Field> {
    Value(Scalar<F>),
    //    Address(usize),
}

impl<F: Field> Cell<F> {
    pub fn value(self) -> Result<Scalar<F>, RuntimeError> {
        match self {
            Cell::Value(value) => Ok(value),
            //            Cell::Address(_) => Err(RuntimeError::UnexpectedNonValueType),
        }
    }
}

impl<F: Field> From<Scalar<F>> for Cell<F> {
    fn from(scalar: Scalar<F>) -> Self {
        Cell::Value(scalar)
    }
}

pub mod arithmetic;
pub mod arrays;
pub mod auto_const;
pub mod boolean;
pub mod comparison;
mod conditional_select;
mod expression;
pub mod types;

pub use arithmetic::*;
pub use arrays::*;
pub use boolean::*;
pub use comparison::*;
pub use conditional_select::*;
pub use expression::*;
pub use types::*;

mod misc;
mod scalar;
pub mod utils;
pub use scalar::*;

use algebra::Field;
use r1cs_core::ConstraintSystem;

pub use misc::*;

use crate::core::RuntimeError;

pub trait Gadget<F: Field> {
    type Input;
    type Output;

    /// Synthesize circuit for the function.
    fn synthesize<CS: ConstraintSystem<F>>(
        &self,
        cs: CS,
        input: Self::Input,
    ) -> Result<Self::Output, RuntimeError>;

    fn input_from_vec(input: &[Scalar<F>]) -> Result<Self::Input, RuntimeError>;
    fn output_into_vec(output: Self::Output) -> Vec<Scalar<F>>;

    fn synthesize_vec<CS: ConstraintSystem<F>>(
        &self,
        cs: CS,
        input: &[Scalar<F>],
    ) -> Result<Vec<Scalar<F>>, RuntimeError> {
        let input = Self::input_from_vec(input)?;
        let output = self.synthesize(cs, input)?;
        Ok(Self::output_into_vec(output))
    }
}

pub use misc::Gadgets;

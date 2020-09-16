pub mod arithmetic;
pub mod arrays;
pub mod auto_const;
pub mod boolean;
pub mod comparison;
mod conditional_select;
pub mod expression;
mod misc;
mod scalar;
pub mod types;
pub mod utils;

pub use arithmetic::*;
pub use arrays::*;
pub use boolean::*;
pub use comparison::*;
pub use conditional_select::*;
pub use expression::*;
pub use misc::*;
pub use scalar::*;
pub use types::*;

use crate::Engine;
use crate::core::RuntimeError;
use r1cs_core::ConstraintSystem;

pub trait Gadget<E: Engine> {
    type Input;
    type Output;

    /// Synthesize circuit for the function.
    fn synthesize<CS: ConstraintSystem<E::Fr>>(
        &self,
        cs: CS,
        input: Self::Input,
    ) -> Result<Self::Output, RuntimeError>;

    fn input_from_vec(input: &[Scalar<E>]) -> Result<Self::Input, RuntimeError>;
    fn output_into_vec(output: Self::Output) -> Vec<Scalar<E>>;

    fn synthesize_vec<CS: ConstraintSystem<E::Fr>>(
        &self,
        cs: CS,
        input: &[Scalar<E>],
    ) -> Result<Vec<Scalar<E>>, RuntimeError> {
        let input = Self::input_from_vec(input)?;
        let output = self.synthesize(cs, input)?;
        Ok(Self::output_into_vec(output))
    }
}

pub use misc::Gadgets;

mod cell;
mod data_stack;
mod evaluation_stack;

pub use cell::*;
pub use data_stack::*;
pub use evaluation_stack::*;

use crate::gadgets::Scalar;
use algebra::Field;
use std::fmt;

#[derive(Debug)]
pub struct Loop {
    pub first_instruction_index: usize,
    pub iterations_left: usize,
}

#[derive(Debug)]
pub struct Branch<F: Field> {
    pub condition: Scalar<F>,
    /// False if there is only one case (If-Endif), true if two cases (If-Else-Endif).
    pub is_full: bool,
}

#[derive(Debug)]
pub enum Block<F: Field> {
    Loop(Loop),
    Branch(Branch<F>),
}

#[derive(Debug)]
pub struct FunctionFrame<F: Field> {
    pub blocks: Vec<Block<F>>,
    pub return_address: usize,
    pub stack_frame_begin: usize,
    pub stack_frame_end: usize,
}

#[derive(Debug)]
pub struct State<F: Field> {
    pub instruction_counter: usize,
    pub evaluation_stack: EvaluationStack<F>,
    pub data_stack: DataStack<F>,
    pub conditions_stack: Vec<Scalar<F>>,
    pub frames_stack: Vec<FunctionFrame<F>>,
}

impl<F: Field> FunctionFrame<F> {
    pub fn new(data_stack_address: usize, return_address: usize) -> Self {
        Self {
            blocks: vec![],
            return_address,
            stack_frame_begin: data_stack_address,
            stack_frame_end: data_stack_address,
        }
    }
}

impl<F: Field> fmt::Display for State<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.evaluation_stack)?;
        writeln!(
            f,
            "Data Stack Offset: {}\n",
            self.frames_stack.last().unwrap().stack_frame_begin
        )?;
        writeln!(f, "{}", self.data_stack)?;

        Ok(())
    }
}

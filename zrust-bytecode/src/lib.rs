//!
//! ZRust bytecode library.
//!

pub mod instructions;
mod vlq;

use std::fmt::Debug;
use crate::instructions::*;
use std::cmp;

#[derive(Debug)]
pub enum InstructionCode {
    NoOperation = 0,

    // Stack
    Pop = 1,
    Push = 2,
    Copy = 99,

    // Arithmetic
    Add = 3,
    Sub = 4,
    Mul,
    Div,
    Rem,

    // Boolean
    Not,
    And,
    Or,
    Xor,

    // Comparison
    Lt,
    Le,
    Eq,
    Ne,
    Ge,
    Gt,
}

pub trait Instruction: Debug {
    fn to_assembly(&self) -> String;
    fn code(&self) -> InstructionCode;
    fn encode(&self) -> Vec<u8>;
}

#[derive(Debug)]
pub enum DecodingError {
    UnexpectedEOF,
    UnknownInstructionCode(u8),
}

pub fn decode_all(bytes: &[u8]) -> Result<Vec<Box<dyn Instruction>>, DecodingError> {
    let mut instructions = Vec::new();

    let mut offset = 0;
    while offset < bytes.len() {
        match decode_instruction(&bytes[offset..]) {
            Ok((instr, len)) => {
                instructions.push(instr);
                offset += len;
            },
            Err(err) => {
                let last = cmp::min(bytes.len(), offset + 10);
                log::warn!("failed to decode bytes {:?} at offset {}", &bytes[offset..last], offset);
                return Err(err);
            }
        };
    }

    Ok(instructions)
}

pub fn decode_instruction(bytes: &[u8]) -> Result<(Box<dyn Instruction>, usize), DecodingError> {
    if bytes.len() < 1 {
        return Err(DecodingError::UnexpectedEOF);
    }

    match bytes[0] {
        x if x == InstructionCode::NoOperation as u8 =>
            NoOperation::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Push as u8 =>
            Push::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Pop as u8 =>
            Pop::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Copy as u8 =>
            Copy::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Add as u8 =>
            Add::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Sub as u8 =>
            Sub::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Mul as u8 =>
            Mul::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Div as u8 =>
            Div::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Rem as u8 =>
            Rem::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Not as u8 =>
            Not::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::And as u8 =>
            And::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Or as u8 =>
            Or::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Xor as u8 =>
            Xor::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        code => Err(DecodingError::UnknownInstructionCode(code))
    }
}

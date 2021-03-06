//!
//! The semantic analyzer scope error.
//!

use crate::lexical::token::location::Location;

#[derive(Debug, PartialEq)]
pub enum Error {
    ItemUndeclared {
        name: String,
    },
    ItemRedeclared {
        name: String,
        reference: Option<Location>,
    },
    ItemIsNotNamespace {
        name: String,
    },
}

//!
//! A semantic analyzer test.
//!

#![cfg(test)]

use crate::error::Error;
use crate::lexical::Location;
use crate::semantic::Error as SemanticError;

#[test]
fn test() {
    let input = r#"
fn main() {
    let scrutinee = 42;
    let result = match scrutinee {
        1 => 10,
        _ => 101,
        2 => 20,
    };
}
"#;

    let expected = Err(Error::Semantic(SemanticError::MatchBranchUnreachable {
        location: Location::new(7, 9),
    }));

    let result = super::compile_entry_point(input);

    assert_eq!(result, expected);
}
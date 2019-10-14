//!
//! The interpreter error.
//!

use failure::Fail;

use parser::Literal;
use parser::Location;
use parser::TypeVariant;

use crate::element::ArrayError;
use crate::element::Error as ElementError;
use crate::element::IntegerError;
use crate::element::StructureError;
use crate::element::Value;
use crate::scope::Error as ScopeError;

#[derive(Debug, Fail, PartialEq)]
pub enum Error {
    #[fail(display = "{} element: {}", _0, _1)]
    Element(Location, ElementError),
    #[fail(display = "{} scope: {}", _0, _1)]
    Scope(Location, ScopeError),
    #[fail(display = "{} literal cannot be evaluated: {}", _0, _1)]
    LiteralCannotBeEvaluated(Location, Literal),
    #[fail(display = "{} array literal: {}", _0, _1)]
    ArrayLiteral(Location, ArrayError),
    #[fail(display = "{} structure literal: {}", _0, _1)]
    StructureLiteral(Location, StructureError),
    #[fail(
        display = "{} conditional expected a boolean expression, but got '{}'",
        _0, _1
    )]
    ConditionalExpectedBooleanExpression(Location, Value),
    #[fail(
        display = "{} let declaration invalid type: '{}' cannot be casted to '{}'",
        _0, _1, _2
    )]
    LetInvalidType(Location, TypeVariant, TypeVariant),
    #[fail(display = "{} let declaration implicit casting: {}", _0, _1)]
    LetImplicitCasting(Location, IntegerError),
    #[fail(
        display = "{} the require '{}' expected a boolean expression, but got '{}'",
        _0, _1, _2
    )]
    RequireExpectedBooleanExpression(Location, String, Value),
    #[fail(display = "{} the require '{}' failed", _0, _1)]
    RequireFailed(Location, String),
    #[fail(display = "{} loop iterator: {}", _0, _1)]
    LoopIterator(Location, IntegerError),
    #[fail(
        display = "{} loop while condition expected a boolean expression, but got '{}'",
        _0, _1
    )]
    LoopWhileExpectedBooleanExpression(Location, Value),
}

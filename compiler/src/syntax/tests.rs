//!
//! The syntax tests.
//!

#![cfg(test)]

use crate::lexical;
use crate::lexical::IntegerLiteral;
use crate::lexical::Location;
use crate::lexical::TokenStream;
use crate::syntax::CircuitProgram;
use crate::syntax::Error;
use crate::syntax::Expression;
use crate::syntax::Identifier;
use crate::syntax::Input;
use crate::syntax::LetStatement;
use crate::syntax::Literal;
use crate::syntax::OperatorExpression;
use crate::syntax::OperatorExpressionElement;
use crate::syntax::OperatorExpressionObject;
use crate::syntax::OperatorExpressionOperand;
use crate::syntax::OperatorExpressionOperator;
use crate::syntax::Parser;
use crate::syntax::Statement;
use crate::syntax::Type;
use crate::syntax::TypeVariant;
use crate::syntax::Witness;

#[test]
fn ok() {
    let code = r#"
inputs {
    a: uint8;
}

witness {
    b: int248;
}

let mut c: uint232 = 2 + 2;
"#;

    let expected: CircuitProgram = CircuitProgram {
        inputs: vec![Input::new(
            Location::new(3, 5),
            Identifier::new(Location::new(3, 5), "a".to_owned()),
            Type::new(Location::new(3, 8), TypeVariant::Uint { bitlength: 8 }),
        )],
        witnesses: vec![Witness::new(
            Location::new(7, 5),
            Identifier::new(Location::new(7, 5), "b".to_owned()),
            Type::new(Location::new(7, 8), TypeVariant::Int { bitlength: 248 }),
        )],
        statements: vec![Statement::Let(LetStatement {
            location: Location::new(10, 1),
            identifier: Identifier::new(Location::new(10, 9), "c".to_owned()),
            r#type: Some(Type::new(
                Location::new(10, 12),
                TypeVariant::Uint { bitlength: 232 },
            )),
            expression: Expression::Operator(OperatorExpression::new(
                Location::new(10, 22),
                vec![
                    OperatorExpressionElement::new(
                        Location::new(10, 22),
                        OperatorExpressionObject::Operand(OperatorExpressionOperand::Literal(
                            Literal::new(
                                Location::new(10, 22),
                                lexical::Literal::Integer(IntegerLiteral::decimal("2".to_owned())),
                            ),
                        )),
                    ),
                    OperatorExpressionElement::new(
                        Location::new(10, 26),
                        OperatorExpressionObject::Operand(OperatorExpressionOperand::Literal(
                            Literal::new(
                                Location::new(10, 26),
                                lexical::Literal::Integer(IntegerLiteral::decimal("2".to_owned())),
                            ),
                        )),
                    ),
                    OperatorExpressionElement::new(
                        Location::new(10, 24),
                        OperatorExpressionObject::Operator(OperatorExpressionOperator::Addition),
                    ),
                ],
            )),
            is_mutable: true,
        })],
    };

    let result: CircuitProgram =
        Parser::parse(TokenStream::new(code.to_owned())).expect("Syntax error");

    assert_eq!(expected, result);
}

#[test]
fn err_unexpected_end() {
    use crate::Error as MainError;

    let code = "inputs";

    let result: Result<CircuitProgram, MainError> =
        Parser::parse(TokenStream::new(code.to_owned()));

    let expected: Result<CircuitProgram, MainError> =
        Err(MainError::Syntax(Error::UnexpectedEnd(Location::new(1, 7))));

    assert_eq!(expected, result);
}

#[test]
fn err_expected() {
    use crate::lexical::Lexeme;
    use crate::lexical::Symbol;
    use crate::Error as MainError;

    let code = "inputs ! ";

    let result: Result<CircuitProgram, MainError> =
        Parser::parse(TokenStream::new(code.to_owned()));

    let expected: Result<CircuitProgram, MainError> = Err(MainError::Syntax(Error::Expected(
        Location::new(1, 8),
        vec!["{"],
        Lexeme::Symbol(Symbol::ExclamationMark),
    )));

    assert_eq!(expected, result);
}

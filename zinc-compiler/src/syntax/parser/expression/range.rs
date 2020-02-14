//!
//! The assignment operand parser.
//!

use std::cell::RefCell;
use std::rc::Rc;

use crate::error::Error;
use crate::lexical::Lexeme;
use crate::lexical::Location;
use crate::lexical::Symbol;
use crate::lexical::Token;
use crate::lexical::TokenStream;
use crate::syntax::Expression;
use crate::syntax::ExpressionBuilder;
use crate::syntax::ExpressionOperator;
use crate::syntax::OrOperandParser;

#[derive(Debug, Clone, Copy)]
pub enum State {
    LogicalOrOperand,
    LogicalOrOperator,
}

impl Default for State {
    fn default() -> Self {
        State::LogicalOrOperand
    }
}

#[derive(Default)]
pub struct Parser {
    state: State,
    builder: ExpressionBuilder,
    operator: Option<(Location, ExpressionOperator)>,
    next: Option<Token>,
}

impl Parser {
    pub fn parse(
        mut self,
        stream: Rc<RefCell<TokenStream>>,
        mut initial: Option<Token>,
    ) -> Result<(Expression, Option<Token>), Error> {
        loop {
            match self.state {
                State::LogicalOrOperand => {
                    let (expression, next) =
                        OrOperandParser::default().parse(stream.clone(), initial.take())?;
                    self.next = next;
                    self.builder.set_location_if_unset(expression.location);
                    self.builder.extend_with_expression(expression);
                    if let Some((location, operator)) = self.operator.take() {
                        self.builder.push_operator(location, operator);
                    }
                    self.state = State::LogicalOrOperator;
                }
                State::LogicalOrOperator => {
                    match crate::syntax::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::DoubleVerticalBar),
                            location,
                        } => {
                            self.operator = Some((location, ExpressionOperator::Or));
                            self.state = State::LogicalOrOperand;
                        }
                        token => return Ok((self.builder.finish(), Some(token))),
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::Parser;
    use crate::lexical;
    use crate::lexical::Lexeme;
    use crate::lexical::Location;
    use crate::lexical::Token;
    use crate::lexical::TokenStream;
    use crate::syntax::BooleanLiteral;
    use crate::syntax::Expression;
    use crate::syntax::ExpressionElement;
    use crate::syntax::ExpressionObject;
    use crate::syntax::ExpressionOperand;
    use crate::syntax::ExpressionOperator;

    #[test]
    fn ok() {
        let input = r#"true || false"#;

        let expected = Ok((
            Expression::new(
                Location::new(1, 1),
                vec![
                    ExpressionElement::new(
                        Location::new(1, 1),
                        ExpressionObject::Operand(ExpressionOperand::LiteralBoolean(
                            BooleanLiteral::new(Location::new(1, 1), lexical::BooleanLiteral::True),
                        )),
                    ),
                    ExpressionElement::new(
                        Location::new(1, 9),
                        ExpressionObject::Operand(ExpressionOperand::LiteralBoolean(
                            BooleanLiteral::new(
                                Location::new(1, 9),
                                lexical::BooleanLiteral::False,
                            ),
                        )),
                    ),
                    ExpressionElement::new(
                        Location::new(1, 6),
                        ExpressionObject::Operator(ExpressionOperator::Or),
                    ),
                ],
            ),
            Some(Token::new(Lexeme::Eof, Location::new(1, 14))),
        ));

        let result = Parser::default().parse(Rc::new(RefCell::new(TokenStream::new(input))), None);

        assert_eq!(result, expected);
    }
}

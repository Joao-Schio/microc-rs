use crate::{
    ast::expression::Expression,
    token::{Token, TokenType},
};

#[derive(Debug, PartialEq, Eq)]
pub enum ParserError {
    UnexpectedToken {
        expected: &'static str,
        found: TokenType,
    },
}

pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        let expression = match self.tokens[self.current].get_tok_type() {
            TokenType::IntegerConst(value) => Expression::Integer(*value),
            TokenType::Id => {
                Expression::Identifier(self.tokens[self.current].get_lexema().to_owned())
            }
            found => {
                return Err(ParserError::UnexpectedToken {
                    expected: "expression",
                    found: *found,
                });
            }
        };

        self.current += 1;
        Ok(expression)
    }
}

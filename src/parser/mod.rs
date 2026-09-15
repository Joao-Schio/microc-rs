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
    fn current(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn advance(&mut self) {
        self.current += 1;
    }

    pub fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        self.parse_factor()
    }

    pub fn parse_factor(&mut self) -> Result<Expression, ParserError> {
        let token = self.current();
        let expression = match *token.get_tok_type() {
            TokenType::IntegerConst(value) => Expression::Integer(value),
            TokenType::Id => Expression::Identifier(token.get_lexema().to_owned()),
            TokenType::CharConst => Expression::Char(
                *token
                    .get_lexema()
                    .first()
                    .expect("char const must have a byte at index 0"),
            ),
            found => {
                return Err(ParserError::UnexpectedToken {
                    expected: "expression",
                    found: found,
                });
            }
        };
        self.advance();
        Ok(expression)
    }
}

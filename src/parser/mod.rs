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
    fn expect(
        &mut self,
        expected: TokenType,
        expected_description: &'static str,
    ) -> Result<(), ParserError> {
        let found = *self.current().get_tok_type();

        if found != expected {
            return Err(ParserError::UnexpectedToken {
                expected: expected_description,
                found,
            });
        }

        self.advance();
        Ok(())
    }
    
    pub fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        self.parse_factor()
    }

    fn parse_factor(&mut self) -> Result<Expression, ParserError> {
        match *self.current().get_tok_type() {
            TokenType::IntegerConst(value) => {
                self.advance();
                Ok(Expression::Integer(value))
            }

            TokenType::Id => {
                let identifier = self.current().get_lexema().to_owned();
                self.advance();

                Ok(Expression::Identifier(identifier))
            }

            TokenType::CharConst => {
                let value = *self
                    .current()
                    .get_lexema()
                    .first()
                    .expect("char const must have a byte at index 0");

                self.advance();

                Ok(Expression::Char(value))
            }

            TokenType::Lparen => {
                self.advance();

                let expression = self.parse_expression()?;

                self.expect(TokenType::Rparen, "')'")?;

                Ok(expression)
            }

            found => Err(ParserError::UnexpectedToken {
                expected: "expression",
                found,
            }),
        }
    }
}

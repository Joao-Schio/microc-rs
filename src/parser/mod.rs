use crate::{
    ast::expression::{BinaryOp, Expression, UnaryOp},
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
        self.parse_logical()
    }

    fn parse_unary(&mut self, op: UnaryOp) -> Result<Expression, ParserError> {
        self.advance();

        let expression = self.parse_factor()?;

        Ok(Expression::Unary {
            op,
            expression: Box::new(expression),
        })
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
                if *self.current().get_tok_type() != TokenType::LBrace {
                    return Ok(Expression::Identifier(identifier));
                }
                let expression = self.parse_expression()?;
                self.advance();
                if *self.current().get_tok_type() != TokenType::RBrace {
                    return Err(ParserError::UnexpectedToken {
                        expected: "]",
                        found: self.current().get_tok_type().clone(),
                    });
                }
                Ok(Expression::ArrayAccess {
                    array: identifier,
                    index: Box::new(expression),
                })
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

            TokenType::Not => self.parse_unary(UnaryOp::Not),

            TokenType::Minus => self.parse_unary(UnaryOp::Negate),

            found => Err(ParserError::UnexpectedToken {
                expected: "expression",
                found,
            }),
        }
    }

    fn parse_term(&mut self) -> Result<Expression, ParserError> {
        let mut expression = self.parse_factor()?;

        loop {
            let op = match *self.current().get_tok_type() {
                TokenType::Mul => BinaryOp::Multiply,
                TokenType::Div => BinaryOp::Divide,
                TokenType::Mod => BinaryOp::Modulo,
                _ => break,
            };

            self.advance();

            let right = self.parse_factor()?;

            expression = Expression::Binary {
                left: Box::new(expression),
                op,
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    fn parse_arithmetic(&mut self) -> Result<Expression, ParserError> {
        let mut expression = self.parse_term()?;

        loop {
            let op = match *self.current().get_tok_type() {
                TokenType::Plus => BinaryOp::Add,
                TokenType::Minus => BinaryOp::Subtract,
                _ => break,
            };

            self.advance();

            let right = self.parse_term()?;

            expression = Expression::Binary {
                left: Box::new(expression),
                op,
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    fn parse_relational(&mut self) -> Result<Expression, ParserError> {
        let left = self.parse_arithmetic()?;

        let op = match *self.current().get_tok_type() {
            TokenType::Eq => BinaryOp::Equal,
            TokenType::Neq => BinaryOp::NotEqual,
            TokenType::Lt => BinaryOp::Less,
            TokenType::Leq => BinaryOp::LessEqual,
            TokenType::Gt => BinaryOp::Greater,
            TokenType::Geq => BinaryOp::GreaterEqual,
            _ => return Ok(left),
        };

        self.advance();

        let right = self.parse_arithmetic()?;

        Ok(Expression::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        })
    }

    fn parse_logical(&mut self) -> Result<Expression, ParserError> {
        let mut expression = self.parse_relational()?;

        loop {
            let op = match *self.current().get_tok_type() {
                TokenType::And => BinaryOp::And,
                TokenType::Or => BinaryOp::Or,
                _ => break,
            };

            self.advance();

            let right = self.parse_relational()?;

            expression = Expression::Binary {
                left: Box::new(expression),
                op,
                right: Box::new(right),
            };
        }

        Ok(expression)
    }
}

use crate::{
    ast::expression::{BinaryOp, Expression, UnaryOp},
    token::TokenType,
};

use super::{ParserContext, ParserError};

pub trait TExprParser {
    fn parse_expression(
        &mut self,
        context: &mut ParserContext<'_>,
    ) -> Result<Expression, ParserError>;
}

#[derive(Default)]
pub struct ExprParser;

impl ExprParser {
    #[inline]
    fn parse_unary(
        &mut self,
        context: &mut ParserContext<'_>,
        op: UnaryOp,
    ) -> Result<Expression, ParserError> {
        context.advance();

        let expression = self.parse_factor(context)?;

        Ok(Expression::Unary {
            op,
            expression: Box::new(expression),
        })
    }

    #[inline]
    fn parse_factor(&mut self, context: &mut ParserContext<'_>) -> Result<Expression, ParserError> {
        match *context.current().get_tok_type() {
            TokenType::IntegerConst(value) => {
                context.advance();
                Ok(Expression::Integer(value))
            }

            TokenType::Id => {
                let identifier = context.current().get_lexema().to_owned();
                context.advance();

                if *context.current().get_tok_type() != TokenType::LBracket {
                    return Ok(Expression::Identifier(identifier));
                }

                context.advance();
                let index = self.parse_expression(context)?;
                context.expect(TokenType::RBracket, "']'")?;

                Ok(Expression::ArrayAccess {
                    array: identifier,
                    index: Box::new(index),
                })
            }

            TokenType::CharConst => {
                let value = *context
                    .current()
                    .get_lexema()
                    .first()
                    .expect("char const must have a byte at index 0");

                context.advance();

                Ok(Expression::Char(value))
            }

            TokenType::Lparen => {
                context.advance();

                let expression = self.parse_expression(context)?;

                context.expect(TokenType::Rparen, "')'")?;

                Ok(expression)
            }

            TokenType::Not => self.parse_unary(context, UnaryOp::Not),

            TokenType::Minus => self.parse_unary(context, UnaryOp::Negate),

            found => Err(ParserError::UnexpectedToken {
                expected: "expression",
                found,
                line: context.current().get_linha(),
            }),
        }
    }

    #[inline]
    fn parse_term(&mut self, context: &mut ParserContext<'_>) -> Result<Expression, ParserError> {
        let mut expression = self.parse_factor(context)?;

        loop {
            let op = match *context.current().get_tok_type() {
                TokenType::Mul => BinaryOp::Multiply,
                TokenType::Div => BinaryOp::Divide,
                TokenType::Mod => BinaryOp::Modulo,
                _ => break,
            };

            context.advance();

            let right = self.parse_factor(context)?;

            expression = Expression::Binary {
                left: Box::new(expression),
                op,
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    #[inline]
    fn parse_arithmetic(
        &mut self,
        context: &mut ParserContext<'_>,
    ) -> Result<Expression, ParserError> {
        let mut expression = self.parse_term(context)?;

        loop {
            let op = match *context.current().get_tok_type() {
                TokenType::Plus => BinaryOp::Add,
                TokenType::Minus => BinaryOp::Subtract,
                _ => break,
            };

            context.advance();

            let right = self.parse_term(context)?;

            expression = Expression::Binary {
                left: Box::new(expression),
                op,
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    #[inline]
    fn parse_relational(
        &mut self,
        context: &mut ParserContext<'_>,
    ) -> Result<Expression, ParserError> {
        let left = self.parse_arithmetic(context)?;

        let op = match *context.current().get_tok_type() {
            TokenType::Eq => BinaryOp::Equal,
            TokenType::Neq => BinaryOp::NotEqual,
            TokenType::Lt => BinaryOp::Less,
            TokenType::Leq => BinaryOp::LessEqual,
            TokenType::Gt => BinaryOp::Greater,
            TokenType::Geq => BinaryOp::GreaterEqual,
            _ => return Ok(left),
        };

        context.advance();

        let right = self.parse_arithmetic(context)?;

        Ok(Expression::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        })
    }

    #[inline]
    fn parse_logical(
        &mut self,
        context: &mut ParserContext<'_>,
    ) -> Result<Expression, ParserError> {
        let mut expression = self.parse_relational(context)?;

        loop {
            let op = match *context.current().get_tok_type() {
                TokenType::And => BinaryOp::And,
                TokenType::Or => BinaryOp::Or,
                _ => break,
            };

            context.advance();

            let right = self.parse_relational(context)?;

            expression = Expression::Binary {
                left: Box::new(expression),
                op,
                right: Box::new(right),
            };
        }

        Ok(expression)
    }
}

impl TExprParser for ExprParser {
    fn parse_expression(
        &mut self,
        context: &mut ParserContext<'_>,
    ) -> Result<Expression, ParserError> {
        self.parse_logical(context)
    }
}

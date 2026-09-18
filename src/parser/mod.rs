mod expression;

use std::{error::Error, fmt};

use crate::{
    ast::{
        expression::Expression,
        statement::{AssignmentTarget, Statement},
    },
    token::{
        Token,
        TokenType::{self, SemiColon},
    },
};

pub use expression::{ExprParser, TExprParser};

#[derive(Debug, PartialEq, Eq)]
pub enum ParserError {
    UnexpectedToken {
        expected: &'static str,
        found: TokenType,
        line: usize,
    },
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedToken {
                expected,
                found,
                line,
            } => write!(f, "expected {expected}, found {found:?} at line {line}"),
        }
    }
}

impl Error for ParserError {}

pub struct ParserContext<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> ParserContext<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, current: 0 }
    }

    pub(crate) fn current(&self) -> &Token {
        &self.tokens[self.current]
    }

    pub(crate) fn advance(&mut self) {
        self.current += 1;
    }

    pub(crate) fn expect(
        &mut self,
        expected: TokenType,
        expected_description: &'static str,
    ) -> Result<(), ParserError> {
        let found = *self.current().get_tok_type();
        if found != expected {
            return Err(ParserError::UnexpectedToken {
                expected: expected_description,
                found,
                line: self.current().get_linha(),
            });
        }

        self.advance();
        Ok(())
    }
}

pub struct Parser<'a, E: TExprParser = ExprParser> {
    context: ParserContext<'a>,
    expr_parser: E,
}

impl<'a> Parser<'a, ExprParser> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self::with_expr_parser(tokens, ExprParser)
    }
}

impl<'a, E: TExprParser> Parser<'a, E> {
    pub fn with_expr_parser(tokens: &'a [Token], expr_parser: E) -> Self {
        Self {
            context: ParserContext::new(tokens),
            expr_parser,
        }
    }

    pub fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        self.expr_parser.parse_expression(&mut self.context)
    }

    fn parse_assignment_target(&mut self) -> Result<AssignmentTarget, ParserError> {
        let identifier = self.context.current().get_lexema().to_owned();
        self.context.expect(TokenType::Id, "Id")?;
        match *self.context.current().get_tok_type() {
            TokenType::LBracket => {
                self.context.advance();
                let expr = self.expr_parser.parse_expression(&mut self.context)?;
                self.context.expect(TokenType::RBracket, "']'")?;
                Ok(AssignmentTarget::ArrayElement {
                    array: identifier,
                    index: Box::new(expr),
                })
            }
            _ => Ok(AssignmentTarget::Identifier(identifier)),
        }
    }

    fn parse_assignment(&mut self) -> Result<Statement, ParserError> {
        let target = self.parse_assignment_target()?;
        self.context.expect(TokenType::Assign, "'='")?;
        let value = self.expr_parser.parse_expression(&mut self.context)?;
        self.context.expect(TokenType::SemiColon, "';'")?;
        Ok(Statement::Assignment { target, value })
    }

    fn parse_return(&mut self) -> Result<Statement, ParserError> {
        self.context.expect(TokenType::Return, "'return'")?;

        let value = match *self.context.current().get_tok_type() {
            TokenType::SemiColon => None,
            _ => Some(self.expr_parser.parse_expression(&mut self.context)?),
        };
        self.context.expect(TokenType::SemiColon, "';'")?;
        Ok(Statement::Return { value })
    }

    pub fn parse_statement(&mut self) -> Result<Statement, ParserError> {
        match *self.context.current().get_tok_type() {
            TokenType::Id => self.parse_assignment(),
            TokenType::Return => self.parse_return(),
            found => Err(ParserError::UnexpectedToken {
                expected: "statement",
                found,
                line: self.context.current().get_linha(),
            }),
        }
    }
}

mod expression;

use std::{error::Error, fmt};

use crate::{
    ast::expression::Expression,
    token::{Token, TokenType},
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
}

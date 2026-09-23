mod expression;
mod statement;

use std::{error::Error, fmt, iter::Peekable, vec::IntoIter};

use crate::{
    ast::{
        expression::Expression,
        statement::{Statement, VariableDeclaration},
    },
    token::{Token, TokenType},
};

pub use expression::{ExprParser, TExprParser};
pub use statement::{StatementParser, TStatementParser};

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

pub struct ParserContext {
    tokens: Peekable<IntoIter<Token>>,
}

impl ParserContext {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens.into_iter().peekable(),
        }
    }

    pub(crate) fn current(&mut self) -> &Token {
        self.tokens
            .peek()
            .expect("parser token stream must contain an EOF token")
    }

    pub(crate) fn advance(&mut self) {
        let _ = self.tokens.next();
    }

    pub(crate) fn expect(&mut self, expected: TokenType) -> Result<Token, ParserError> {
        let expected_description = expected
            .get_expected_lexeme()
            .expect("ParserContext::expect requires a token type with a fixed expected lexeme");

        self.expect_matching(expected_description, |found| found == &expected)
    }

    pub(crate) fn expect_matching<F>(
        &mut self,
        expected_description: &'static str,
        predicate: F,
    ) -> Result<Token, ParserError>
    where
        F: FnOnce(&TokenType) -> bool,
    {
        let matches = {
            let current = self.current();
            predicate(current.token_type())
        };

        if !matches {
            let current = self.current();
            return Err(ParserError::UnexpectedToken {
                expected: expected_description,
                found: current.token_type().clone(),
                line: current.line(),
            });
        }

        Ok(self
            .tokens
            .next()
            .expect("peeked parser token must still be available"))
    }
}

pub struct Parser<E: TExprParser = ExprParser, S: TStatementParser<E> = StatementParser> {
    context: ParserContext,
    expr_parser: E,
    statement_parser: S,
}

impl Parser<ExprParser, StatementParser> {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self::with_parsers(tokens, ExprParser, StatementParser)
    }
}

impl<E: TExprParser> Parser<E, StatementParser> {
    pub fn with_expr_parser(tokens: Vec<Token>, expr_parser: E) -> Self {
        Self::with_parsers(tokens, expr_parser, StatementParser)
    }

    pub fn parse_declarations(&mut self) -> Result<Vec<VariableDeclaration>, ParserError> {
        self.statement_parser.parse_declarations(&mut self.context)
    }

    pub fn parse_statement_block(&mut self) -> Result<Vec<Statement>, ParserError> {
        self.statement_parser
            .parse_statement_block(&mut self.context, &mut self.expr_parser)
    }

    pub fn parse_block(&mut self) -> Result<Statement, ParserError> {
        self.statement_parser
            .parse_block(&mut self.context, &mut self.expr_parser)
    }
}

impl<E, S> Parser<E, S>
where
    E: TExprParser,
    S: TStatementParser<E>,
{
    pub fn with_parsers(tokens: Vec<Token>, expr_parser: E, statement_parser: S) -> Self {
        Self {
            context: ParserContext::new(tokens),
            expr_parser,
            statement_parser,
        }
    }

    pub fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        self.expr_parser.parse_expression(&mut self.context)
    }

    pub fn parse_statement(&mut self) -> Result<Statement, ParserError> {
        self.statement_parser
            .parse_statement(&mut self.context, &mut self.expr_parser)
    }
}

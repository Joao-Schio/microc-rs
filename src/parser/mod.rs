mod expression;

use std::{error::Error, fmt, iter::Peekable, vec::IntoIter};

use crate::{
    ast::{
        expression::Expression,
        statement::{AssignmentTarget, PrintContent, Statement},
    },
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

    pub(crate) fn expect(
        &mut self,
        expected: TokenType,
        expected_description: &'static str,
    ) -> Result<Token, ParserError> {
        let (found, line) = {
            let current = self.current();
            (*current.get_tok_type(), current.get_linha())
        };

        if found != expected {
            return Err(ParserError::UnexpectedToken {
                expected: expected_description,
                found,
                line,
            });
        }

        Ok(self
            .tokens
            .next()
            .expect("peeked parser token must still be available"))
    }
}

pub struct Parser<E: TExprParser = ExprParser> {
    context: ParserContext,
    expr_parser: E,
}

impl Parser<ExprParser> {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self::with_expr_parser(tokens, ExprParser)
    }
}

impl<E: TExprParser> Parser<E> {
    pub fn with_expr_parser(tokens: Vec<Token>, expr_parser: E) -> Self {
        Self {
            context: ParserContext::new(tokens),
            expr_parser,
        }
    }

    pub fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        self.expr_parser.parse_expression(&mut self.context)
    }

    fn parse_assignment_target(&mut self) -> Result<AssignmentTarget, ParserError> {
        let identifier = self.context.expect(TokenType::Id, "Id")?.into_lexeme();

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

    fn parse_print(&mut self) -> Result<Statement, ParserError> {
        self.context.expect(TokenType::Print, "'print'")?;
        self.context.expect(TokenType::Lparen, "'('")?;

        let content = match *self.context.current().get_tok_type() {
            TokenType::StringConst => {
                let lexeme = self
                    .context
                    .expect(TokenType::StringConst, "string literal")?
                    .into_lexeme();
                PrintContent::StringConst(lexeme)
            }
            _ => PrintContent::Expression(self.expr_parser.parse_expression(&mut self.context)?),
        };

        self.context.expect(TokenType::Rparen, "')'")?;
        self.context.expect(TokenType::SemiColon, "';'")?;
        Ok(Statement::Print { content })
    }

    pub fn parse_statement(&mut self) -> Result<Statement, ParserError> {
        match *self.context.current().get_tok_type() {
            TokenType::Id => self.parse_assignment(),
            TokenType::Return => self.parse_return(),
            TokenType::Print => self.parse_print(),
            found => Err(ParserError::UnexpectedToken {
                expected: "statement",
                found,
                line: self.context.current().get_linha(),
            }),
        }
    }
}

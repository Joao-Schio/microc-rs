mod expression;

use std::{error::Error, fmt, iter::Peekable, vec::IntoIter};

use crate::{
    ast::{
        expression::Expression,
        statement::{
            AssignmentTarget, Block, DataType, PrintContent, Statement, VariableDeclaration,
        },
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
        let token = self
            .context
            .expect_matching("Id", |found| matches!(found, TokenType::Id(_)))?;

        let TokenType::Id(identifier) = token.into_type() else {
            unreachable!("identifier predicate must only accept TokenType::Id")
        };

        match *self.context.current().token_type() {
            TokenType::LBracket => {
                self.context.advance();
                let expr = self.expr_parser.parse_expression(&mut self.context)?;
                self.context.expect(TokenType::RBracket)?;
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
        self.context.expect(TokenType::Assign)?;
        let value = self.expr_parser.parse_expression(&mut self.context)?;
        self.context.expect(TokenType::SemiColon)?;
        Ok(Statement::Assignment { target, value })
    }

    fn parse_return(&mut self) -> Result<Statement, ParserError> {
        self.context.expect(TokenType::Return)?;

        let value = match *self.context.current().token_type() {
            TokenType::SemiColon => None,
            _ => Some(self.expr_parser.parse_expression(&mut self.context)?),
        };
        self.context.expect(TokenType::SemiColon)?;
        Ok(Statement::Return { value })
    }

    fn parse_print(&mut self) -> Result<Statement, ParserError> {
        self.context.expect(TokenType::Print)?;
        self.context.expect(TokenType::Lparen)?;

        let is_string = matches!(
            self.context.current().token_type(),
            TokenType::StringConst(_)
        );

        let content = if is_string {
            let token = self.context.expect_matching("string literal", |found| {
                matches!(found, TokenType::StringConst(_))
            })?;

            let TokenType::StringConst(string) = token.into_type() else {
                unreachable!("string predicate must only accept TokenType::StringConst")
            };

            PrintContent::StringConst(string)
        } else {
            PrintContent::Expression(self.expr_parser.parse_expression(&mut self.context)?)
        };

        self.context.expect(TokenType::Rparen)?;
        self.context.expect(TokenType::SemiColon)?;
        Ok(Statement::Print { content })
    }

    fn parse_empty(&mut self) -> Result<Statement, ParserError> {
        self.context.expect(TokenType::SemiColon)?;
        Ok(Statement::Empty)
    }

    fn parse_if(&mut self) -> Result<Statement, ParserError> {
        self.context.expect(TokenType::If)?;
        self.context.expect(TokenType::Lparen)?;
        let condition = self.expr_parser.parse_expression(&mut self.context)?;
        self.context.expect(TokenType::Rparen)?;
        let then_branch = Box::new(self.parse_statement()?);
        let else_branch = match self.context.current().token_type() {
            TokenType::Else => {
                self.context.advance();
                Some(Box::new(self.parse_statement()?))
            }
            _ => None,
        };
        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn parse_variable_declarations(
        &mut self,
        data_type: DataType,
    ) -> Result<VariableDeclaration, ParserError> {
        let token = self
            .context
            .expect_matching("Id", |found| matches!(found, TokenType::Id(_)))?;

        let TokenType::Id(identifier) = token.into_type() else {
            unreachable!("identifier predicate must only accept TokenType::Id")
        };

        match self.context.current().token_type() {
            TokenType::LBracket => {
                self.context.expect(TokenType::LBracket)?;

                let token = self.context.expect_matching("IntegerConst", |found| {
                    matches!(found, TokenType::IntegerConst(_))
                })?;

                let TokenType::IntegerConst(length) = token.into_type() else {
                    unreachable!("integer predicate must only accept TokenType::IntegerConst")
                };

                self.context.expect(TokenType::RBracket)?;
                self.context.expect(TokenType::SemiColon)?;

                Ok(VariableDeclaration::Array {
                    data_type,
                    name: identifier,
                    length,
                })
            }

            TokenType::SemiColon => {
                self.context.expect(TokenType::SemiColon)?;

                Ok(VariableDeclaration::Scalar {
                    data_type,
                    name: identifier,
                })
            }

            found => Err(ParserError::UnexpectedToken {
                expected: "';'",
                found: found.clone(),
                line: self.context.current().line(),
            }),
        }
    }

    pub fn parse_declarations(&mut self) -> Result<Vec<VariableDeclaration>, ParserError> {
        let mut declarations = Vec::new();
        loop {
            match *self.context.current().token_type() {
                TokenType::Char => {
                    self.context.expect(TokenType::Char)?;
                    declarations.push(self.parse_variable_declarations(DataType::Char)?)
                }
                TokenType::Int => {
                    self.context.expect(TokenType::Int)?;
                    declarations.push(self.parse_variable_declarations(DataType::Int)?)
                }
                _ => break,
            }
        }
        Ok(declarations)
    }

    pub fn parse_statement_block(&mut self) -> Result<Vec<Statement>, ParserError> {
        let mut statements = Vec::new();
        loop {
            if *self.context.current().token_type() == TokenType::RBrace {
                break;
            }

            if *self.context.current().token_type() == TokenType::Eof {
                return Err(ParserError::UnexpectedToken {
                    expected: "'}'",
                    found: TokenType::Eof,
                    line: self.context.current().line(),
                });
            }
            let statement = self.parse_statement()?;
            statements.push(statement);
        }
        Ok(statements)
    }

    pub fn parse_block(&mut self) -> Result<Statement, ParserError> {
        self.context.expect(TokenType::LBrace)?;
        let declarations = self.parse_declarations()?;
        let statements = self.parse_statement_block()?;
        self.context.expect(TokenType::RBrace)?;
        Ok(Statement::Block(Block {
            declarations,
            statements,
        }))
    }

    pub fn parse_statement(&mut self) -> Result<Statement, ParserError> {
        match self.context.current().token_type() {
            TokenType::Id(_) => self.parse_assignment(),
            TokenType::Return => self.parse_return(),
            TokenType::Print => self.parse_print(),
            TokenType::SemiColon => self.parse_empty(),
            TokenType::If => self.parse_if(),
            TokenType::LBrace => self.parse_block(),
            found => {
                let found = found.clone();
                let line = self.context.current().line();
                Err(ParserError::UnexpectedToken {
                    expected: "statement",
                    found,
                    line,
                })
            }
        }
    }
}

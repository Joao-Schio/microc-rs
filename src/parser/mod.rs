mod expression;
mod statement;

use std::{error::Error, fmt, iter::Peekable, vec::IntoIter};

use crate::{
    ast::{
        expression::Expression,
        program::{GenericFunction, MainFunction, Parameter, Program},
        statement::{Statement, Type, VariableDeclaration},
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
    InvalidMainReturnType {
        found: Type,
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
            Self::InvalidMainReturnType { found } => {
                write!(f, "main must return int, found {found:?}")
            }
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
            .map(Statement::Block)
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

    fn parse_type(&mut self) -> Result<Type, ParserError> {
        match self.context.current().token_type() {
            TokenType::Int => {
                self.context.expect(TokenType::Int)?;
                Ok(Type::Int)
            }
            TokenType::Char => {
                self.context.expect(TokenType::Char)?;
                Ok(Type::Char)
            }
            found => Err(ParserError::UnexpectedToken {
                expected: "int or char",
                found: found.clone(),
                line: self.context.current().line(),
            }),
        }
    }

    fn parse_name(&mut self) -> Result<Vec<u8>, ParserError> {
        let token = self
            .context
            .expect_matching("Id", |found| matches!(found, TokenType::Id(_)))?;
        let TokenType::Id(identifier) = token.into_type() else {
            unreachable!("identifier predicate must only accept TokenType::Id")
        };
        Ok(identifier)
    }

    fn parse_generic_function(
        &mut self,
        return_type: Type,
    ) -> Result<GenericFunction, ParserError> {
        let name = self.parse_name()?;
        self.context.expect(TokenType::Lparen)?;
        let parameters = self.parse_vec_parameter()?;
        self.context.expect(TokenType::Rparen)?;
        let body = self
            .statement_parser
            .parse_block(&mut self.context, &mut self.expr_parser)?;

        Ok(GenericFunction {
            return_type,
            name,
            parameters,
            body,
        })
    }

    fn parse_main_function(&mut self, return_type: Type) -> Result<MainFunction, ParserError> {
        if return_type != Type::Int {
            return Err(ParserError::InvalidMainReturnType { found: return_type });
        }

        self.context.expect(TokenType::Main)?;
        self.context.expect(TokenType::Lparen)?;
        self.context.expect(TokenType::Rparen)?;
        let body = self
            .statement_parser
            .parse_block(&mut self.context, &mut self.expr_parser)?;

        Ok(MainFunction { body })
    }

    fn parse_function_definition(&mut self) -> Result<Function, ParserError> {
        let return_type = self.parse_type()?;
        let function_name = self.context.current().token_type().clone();

        match function_name {
            TokenType::Id(_) => self
                .parse_generic_function(return_type)
                .map(Function::Generic),
            TokenType::Main => self.parse_main_function(return_type).map(Function::Main),
            found => Err(ParserError::UnexpectedToken {
                expected: "function name or 'main'",
                found,
                line: self.context.current().line(),
            }),
        }
    }

    fn parse_parameter(&mut self) -> Result<Parameter, ParserError> {
        let data_type = self.parse_type()?;
        let name = self.parse_name()?;
        Ok(Parameter { data_type, name })
    }

    fn parse_vec_parameter(&mut self) -> Result<Vec<Parameter>, ParserError> {
        let mut parameters = Vec::new();
        loop {
            if *self.context.current().token_type() == TokenType::Rparen {
                break;
            }
            parameters.push(self.parse_parameter()?);
        }
        Ok(parameters)
    }

    pub fn parse_program(&mut self) -> Result<Program, ParserError> {
        let mut functions = Vec::new();

        loop {
            match self.parse_function_definition()? {
                Function::Generic(function) => functions.push(function),
                Function::Main(main) => {
                    self.context.expect(TokenType::Eof)?;
                    return Ok(Program { functions, main });
                }
            }
        }
    }
}

enum Function {
    Generic(GenericFunction),
    Main(MainFunction),
}

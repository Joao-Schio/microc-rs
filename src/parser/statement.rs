use crate::{
    ast::statement::{
        Assignment, Block, LValue, PrintContent, Statement, Type, VariableDeclaration,
    },
    token::TokenType,
};

use super::{ParserContext, ParserError, expression::TExprParser};

pub trait TStatementParser<E: TExprParser> {
    fn parse_statement(
        &mut self,
        context: &mut ParserContext,
        expr_parser: &mut E,
    ) -> Result<Statement, ParserError>;

    fn parse_block(
        &mut self,
        context: &mut ParserContext,
        expr_parser: &mut E,
    ) -> Result<Block, ParserError> {
        if !matches!(context.current().token_type(), TokenType::LBrace) {
            let current = context.current();
            return Err(ParserError::UnexpectedToken {
                expected: "'{'",
                found: current.token_type().clone(),
                line: current.line(),
            });
        }

        match self.parse_statement(context, expr_parser)? {
            Statement::Block(block) => Ok(block),
            _ => Err(ParserError::ExpectedBlock),
        }
    }
}

#[derive(Default)]
pub struct StatementParser;

impl StatementParser {
    fn parse_lvalue<E: TExprParser>(
        &mut self,
        context: &mut ParserContext,
        expr_parser: &mut E,
    ) -> Result<LValue, ParserError> {
        let token = context.expect_matching("Id", |found| matches!(found, TokenType::Id(_)))?;

        let TokenType::Id(identifier) = token.into_type() else {
            unreachable!("identifier predicate must only accept TokenType::Id")
        };

        match *context.current().token_type() {
            TokenType::LBracket => {
                context.advance();
                let expr = expr_parser.parse_expression(context)?;
                context.expect(TokenType::RBracket)?;
                Ok(LValue::ArrayElement {
                    array: identifier,
                    index: Box::new(expr),
                })
            }
            _ => Ok(LValue::Identifier(identifier)),
        }
    }

    fn parse_assignment<E: TExprParser>(
        &mut self,
        context: &mut ParserContext,
        expr_parser: &mut E,
    ) -> Result<Assignment, ParserError> {
        let target = self.parse_lvalue(context, expr_parser)?;
        context.expect(TokenType::Assign)?;
        let value = expr_parser.parse_expression(context)?;

        Ok(Assignment { target, value })
    }

    fn parse_assignment_statement<E: TExprParser>(
        &mut self,
        context: &mut ParserContext,
        expr_parser: &mut E,
    ) -> Result<Statement, ParserError> {
        let assignment = self.parse_assignment(context, expr_parser)?;
        context.expect(TokenType::SemiColon)?;

        Ok(Statement::Assignment(assignment))
    }

    fn parse_return<E: TExprParser>(
        &mut self,
        context: &mut ParserContext,
        expr_parser: &mut E,
    ) -> Result<Statement, ParserError> {
        context.expect(TokenType::Return)?;

        let value = match *context.current().token_type() {
            TokenType::SemiColon => None,
            _ => Some(expr_parser.parse_expression(context)?),
        };
        context.expect(TokenType::SemiColon)?;
        Ok(Statement::Return { value })
    }

    fn parse_print<E: TExprParser>(
        &mut self,
        context: &mut ParserContext,
        expr_parser: &mut E,
    ) -> Result<Statement, ParserError> {
        context.expect(TokenType::Print)?;
        context.expect(TokenType::Lparen)?;

        let is_string = matches!(context.current().token_type(), TokenType::StringConst(_));

        let content = if is_string {
            let token = context.expect_matching("string literal", |found| {
                matches!(found, TokenType::StringConst(_))
            })?;

            let TokenType::StringConst(string) = token.into_type() else {
                unreachable!("string predicate must only accept TokenType::StringConst")
            };

            PrintContent::StringConst(string)
        } else {
            PrintContent::Expression(expr_parser.parse_expression(context)?)
        };

        context.expect(TokenType::Rparen)?;
        context.expect(TokenType::SemiColon)?;
        Ok(Statement::Print { content })
    }

    fn parse_empty(&mut self, context: &mut ParserContext) -> Result<Statement, ParserError> {
        context.expect(TokenType::SemiColon)?;
        Ok(Statement::Empty)
    }

    fn parse_if<E: TExprParser>(
        &mut self,
        context: &mut ParserContext,
        expr_parser: &mut E,
    ) -> Result<Statement, ParserError> {
        context.expect(TokenType::If)?;
        context.expect(TokenType::Lparen)?;
        let condition = expr_parser.parse_expression(context)?;
        context.expect(TokenType::Rparen)?;
        let then_branch = Box::new(self.parse_statement(context, expr_parser)?);
        let else_branch = match context.current().token_type() {
            TokenType::Else => {
                context.advance();
                Some(Box::new(self.parse_statement(context, expr_parser)?))
            }
            _ => None,
        };
        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn parse_variable_declaration(
        &mut self,
        context: &mut ParserContext,
        data_type: Type,
    ) -> Result<VariableDeclaration, ParserError> {
        let token = context.expect_matching("Id", |found| matches!(found, TokenType::Id(_)))?;

        let TokenType::Id(identifier) = token.into_type() else {
            unreachable!("identifier predicate must only accept TokenType::Id")
        };

        match context.current().token_type() {
            TokenType::LBracket => {
                context.expect(TokenType::LBracket)?;

                let token = context.expect_matching("IntegerConst", |found| {
                    matches!(found, TokenType::IntegerConst(_))
                })?;

                let TokenType::IntegerConst(length) = token.into_type() else {
                    unreachable!("integer predicate must only accept TokenType::IntegerConst")
                };

                context.expect(TokenType::RBracket)?;
                context.expect(TokenType::SemiColon)?;

                Ok(VariableDeclaration::Array {
                    data_type,
                    name: identifier,
                    length,
                })
            }

            TokenType::SemiColon => {
                context.expect(TokenType::SemiColon)?;

                Ok(VariableDeclaration::Scalar {
                    data_type,
                    name: identifier,
                })
            }

            found => Err(ParserError::UnexpectedToken {
                expected: "';'",
                found: found.clone(),
                line: context.current().line(),
            }),
        }
    }

    pub(super) fn parse_declarations(
        &mut self,
        context: &mut ParserContext,
    ) -> Result<Vec<VariableDeclaration>, ParserError> {
        let mut declarations = Vec::new();
        loop {
            match *context.current().token_type() {
                TokenType::Char => {
                    context.expect(TokenType::Char)?;
                    declarations.push(self.parse_variable_declaration(context, Type::Char)?)
                }
                TokenType::Int => {
                    context.expect(TokenType::Int)?;
                    declarations.push(self.parse_variable_declaration(context, Type::Int)?)
                }
                _ => break,
            }
        }
        Ok(declarations)
    }

    pub(super) fn parse_statement_block<E: TExprParser>(
        &mut self,
        context: &mut ParserContext,
        expr_parser: &mut E,
    ) -> Result<Vec<Statement>, ParserError> {
        let mut statements = Vec::new();
        loop {
            if *context.current().token_type() == TokenType::RBrace {
                break;
            }

            if *context.current().token_type() == TokenType::Eof {
                return Err(ParserError::UnexpectedToken {
                    expected: "'}'",
                    found: TokenType::Eof,
                    line: context.current().line(),
                });
            }
            let statement = self.parse_statement(context, expr_parser)?;
            statements.push(statement);
        }
        Ok(statements)
    }

    fn parse_block_inner<E: TExprParser>(
        &mut self,
        context: &mut ParserContext,
        expr_parser: &mut E,
    ) -> Result<Block, ParserError> {
        context.expect(TokenType::LBrace)?;
        let declarations = self.parse_declarations(context)?;
        let statements = self.parse_statement_block(context, expr_parser)?;
        context.expect(TokenType::RBrace)?;
        Ok(Block {
            declarations,
            statements,
        })
    }

    fn parse_block_statement<E: TExprParser>(
        &mut self,
        context: &mut ParserContext,
        expr_parser: &mut E,
    ) -> Result<Statement, ParserError> {
        Ok(Statement::Block(self.parse_block_inner(context, expr_parser)?))
    }

    fn parse_for<E: TExprParser>(
        &mut self,
        context: &mut ParserContext,
        expr_parser: &mut E,
    ) -> Result<Statement, ParserError> {
        context.expect(TokenType::For)?;
        context.expect(TokenType::Lparen)?;

        let initialization = self.parse_assignment(context, expr_parser)?;
        context.expect(TokenType::SemiColon)?;

        let condition = expr_parser.parse_expression(context)?;
        context.expect(TokenType::SemiColon)?;

        let update = self.parse_assignment(context, expr_parser)?;
        context.expect(TokenType::Rparen)?;

        let body = Box::new(self.parse_statement(context, expr_parser)?);

        Ok(Statement::For {
            initialization,
            condition,
            update,
            body,
        })
    }
}

impl<E: TExprParser> TStatementParser<E> for StatementParser {
    fn parse_statement(
        &mut self,
        context: &mut ParserContext,
        expr_parser: &mut E,
    ) -> Result<Statement, ParserError> {
        match context.current().token_type() {
            TokenType::Id(_) => self.parse_assignment_statement(context, expr_parser),
            TokenType::Return => self.parse_return(context, expr_parser),
            TokenType::Print => self.parse_print(context, expr_parser),
            TokenType::SemiColon => self.parse_empty(context),
            TokenType::If => self.parse_if(context, expr_parser),
            TokenType::LBrace => self.parse_block_statement(context, expr_parser),
            TokenType::For => self.parse_for(context, expr_parser),
            found => {
                let found = found.clone();
                let line = context.current().line();
                Err(ParserError::UnexpectedToken {
                    expected: "statement",
                    found,
                    line,
                })
            }
        }
    }

    fn parse_block(
        &mut self,
        context: &mut ParserContext,
        expr_parser: &mut E,
    ) -> Result<Block, ParserError> {
        self.parse_block_inner(context, expr_parser)
    }
}

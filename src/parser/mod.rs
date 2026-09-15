pub mod ast;

use std::char::ParseCharError;

use crate::{parser::ast::expression::Expression, token::{Token, TokenType}};




pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse_expression(&mut self) -> Result<Expression, ParseCharError> {
        let saida = match self.tokens[self.current].get_tok_type() {
            TokenType::IntegerConst(num) => {
                 Ok(Expression::Integer(*num))
            }
            _ => todo!()
        };
        self.current += 1;
        return saida;
    }
}
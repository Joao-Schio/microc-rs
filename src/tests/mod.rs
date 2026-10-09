#[macro_use]
mod contracts;

mod comments;
mod frontend_error;
mod frontend_integration;
mod helpers;
mod lexer;
mod numeric;
mod parser;
mod reserved_words;
pub(crate) mod semantic;
mod token_contract;
mod tokenize;

use std::io::Read;

use crate::{
    ast::program::Program,
    frontend_error::FrontendError,
    lexer::{Lexer, tokenize},
    parser::Parser,
    scanner::{Scanner, ScannerError},
    semantic::{TSemanticAnalyzer, analyzer::SemanticAnalyzer},
};

pub fn analyze_source<R: Read>(reader: R) -> Result<Program, FrontendError> {
    let scanner = Scanner::new(reader).map_err(ScannerError::from)?;

    let mut lexer = Lexer::new(scanner);
    let tokens = tokenize(&mut lexer)?;

    let program = Parser::new(tokens).parse_program()?;

    {
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&program)?;
    }

    Ok(program)
}

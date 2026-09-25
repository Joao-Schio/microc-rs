use std::collections::HashMap;

use crate::ast::{
    program::{Parameter, Program},
    statement::VariableDeclaration,
};
pub mod analyzer;

pub enum SemanticError {
    UndeclaredVariable { name: Vec<u8>, line: usize },
}

pub trait TSemanticAnalyzer<'a> {
    fn analyze(&mut self, program: &'a Program) -> Result<(), SemanticError>;
}
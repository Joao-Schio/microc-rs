use crate::ast::program::Program;

pub mod analyzer;

#[derive(Debug, PartialEq, Eq)]
pub enum SemanticError {
    UndeclaredVariable { name: Vec<u8>, line: usize },
}

pub trait TSemanticAnalyzer<'a> {
    fn analyze(&mut self, program: &'a Program) -> Result<(), SemanticError>;
}

use crate::ast::{Identifier, program::Program};

pub mod analyzer;

#[derive(Debug, PartialEq, Eq)]
pub enum SemanticError {
    UndeclaredVariable(Identifier),
    DuplicateDeclaration(Identifier),
    UndeclaredFunction(Identifier)
}

pub trait TSemanticAnalyzer<'a> {
    fn analyze(&mut self, program: &'a Program) -> Result<(), SemanticError>;
}

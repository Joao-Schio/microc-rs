use crate::ast::{
    Identifier,
    expression::{BinaryOp, UnaryOp},
    program::Program,
    statement::Type,
};

pub mod analyzer;

/// The type of an evaluated expression, including whether it names an array.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ExprType {
    Scalar(Type),
    Array(Type),
}

#[derive(Debug, PartialEq, Eq)]
pub enum SemanticError {
    UndeclaredVariable(Identifier),
    DuplicateDeclaration(Identifier),
    UndeclaredFunction(Identifier),
    NotCallable(Identifier),
    NotAVariable(Identifier),
    NotAnArray(Identifier),
    NotAssignable(Identifier),
    TypeMismatch {
        expected: ExprType,
        actual: ExprType,
        line: usize,
    },
    InvalidBinaryOperands {
        operator: BinaryOp,
        left: ExprType,
        right: ExprType,
        line: usize,
    },
    InvalidUnaryOperand {
        operator: UnaryOp,
        actual: ExprType,
        line: usize,
    },
    ArgumentCountMismatch {
        callee: Identifier,
        expected: usize,
        actual: usize,
    },
    InvalidPrintType {
        actual: ExprType,
        line: usize,
    },
    MissingReturnValue {
        line: usize,
    },
}

pub trait TSemanticAnalyzer<'a> {
    fn analyze(&mut self, program: &'a Program) -> Result<(), SemanticError>;
}

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


impl std::fmt::Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UndeclaredVariable(identifier) => write!(
                f,
                "undeclared variable '{}' at line {}",
                String::from_utf8_lossy(identifier.as_bytes()),
                identifier.line
            ),
            Self::DuplicateDeclaration(identifier) => write!(
                f,
                "duplicate declaration of '{}' at line {}",
                String::from_utf8_lossy(identifier.as_bytes()),
                identifier.line
            ),
            Self::UndeclaredFunction(identifier) => write!(
                f,
                "undeclared function '{}' at line {}",
                String::from_utf8_lossy(identifier.as_bytes()),
                identifier.line
            ),
            Self::NotCallable(identifier) => write!(
                f,
                "'{}' is not callable at line {}",
                String::from_utf8_lossy(identifier.as_bytes()),
                identifier.line
            ),
            Self::NotAVariable(identifier) => write!(
                f,
                "'{}' is not a variable at line {}",
                String::from_utf8_lossy(identifier.as_bytes()),
                identifier.line
            ),
            Self::NotAnArray(identifier) => write!(
                f,
                "'{}' is not an array at line {}",
                String::from_utf8_lossy(identifier.as_bytes()),
                identifier.line
            ),
            Self::NotAssignable(identifier) => write!(
                f,
                "'{}' is not assignable at line {}",
                String::from_utf8_lossy(identifier.as_bytes()),
                identifier.line
            ),
            Self::TypeMismatch {
                expected,
                actual,
                line,
            } => write!(
                f,
                "type mismatch at line {line}: expected {expected:?}, found {actual:?}"
            ),
            Self::InvalidBinaryOperands {
                operator,
                left,
                right,
                line,
            } => write!(
                f,
                "invalid operands for {operator:?} at line {line}: {left:?} and {right:?}"
            ),
            Self::InvalidUnaryOperand {
                operator,
                actual,
                line,
            } => write!(
                f,
                "invalid operand for {operator:?} at line {line}: {actual:?}"
            ),
            Self::ArgumentCountMismatch {
                callee,
                expected,
                actual,
            } => write!(
                f,
                "function '{}' at line {} expects {expected} arguments, found {actual}",
                String::from_utf8_lossy(callee.as_bytes()),
                callee.line
            ),
            Self::InvalidPrintType { actual, line } => {
                write!(f, "invalid print type {actual:?} at line {line}")
            }
            Self::MissingReturnValue { line } => {
                write!(f, "missing return value at line {line}")
            }
        }
    }
}

impl std::error::Error for SemanticError {}

pub trait TSemanticAnalyzer<'a> {
    fn analyze(&mut self, program: &'a Program) -> Result<(), SemanticError>;
}

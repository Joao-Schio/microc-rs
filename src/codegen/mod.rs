mod context;
mod expression;
mod statement;

use std::{error::Error, fmt};

use crate::ast::program::Program;

pub use context::{ControlFlow, FunctionContext, Operand};

#[derive(Debug, PartialEq, Eq)]
pub enum CodegenError {
    Unsupported(&'static str),
    IntegerOutOfRange(i64),
    MissingReturn,
    UnreachableStatement { line: usize },
}

impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported(feature) => {
                write!(f, "LLVM backend does not yet support {feature}")
            }
            Self::IntegerOutOfRange(value) => {
                write!(f, "integer {value} does not fit in i32")
            }
            Self::MissingReturn => write!(f, "function 'main' does not return a value"),
            Self::UnreachableStatement { line } => {
                write!(
                    f,
                    "LLVM backend does not yet handle unreachable statements at line {line}"
                )
            }
        }
    }
}

impl Error for CodegenError {}

/// Lowers a single AST node into the current LLVM function.
pub trait EmitLlvm {
    type Output;

    fn emit_llvm(&self, ctx: &mut FunctionContext) -> Result<Self::Output, CodegenError>;
}

/// Emit the currently supported subset: one main function returning i32.
pub fn generate_ir(program: &Program) -> Result<String, CodegenError> {
    if !program.functions.is_empty() {
        return Err(CodegenError::Unsupported("additional functions"));
    }

    let mut ctx = FunctionContext::new();
    if program.main.body.emit_llvm(&mut ctx)? != ControlFlow::Terminated {
        return Err(CodegenError::MissingReturn);
    }

    Ok(format!(
        "define i32 @main() {{\nentry:\n{}}}\n",
        ctx.into_instructions()
    ))
}

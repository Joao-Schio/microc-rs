use crate::ast::expression::{BinaryOp, Expression, UnaryOp};

use super::{CodegenError, EmitLlvm, FunctionContext, Operand};

impl EmitLlvm for Expression {
    type Output = Operand;

    fn emit_llvm(&self, ctx: &mut FunctionContext) -> Result<Operand, CodegenError> {
        match self {
            Self::Integer(value) => {
                let value = i32::try_from(*value)
                    .map_err(|_| CodegenError::IntegerOutOfRange(*value))?;
                Ok(Operand::Constant(value))
            }
            Self::Unary { op, expression } => {
                if *op != UnaryOp::Negate {
                    return Err(CodegenError::Unsupported("unary operator"));
                }

                let operand = expression.emit_llvm(ctx)?;
                let target = ctx.fresh_register();
                ctx.emit(&format!("{target} = sub i32 0, {operand}"));
                Ok(Operand::Register(target))
            }
            Self::Binary { left, op, right } => {
                let opcode = match op {
                    BinaryOp::Add => "add",
                    BinaryOp::Subtract => "sub",
                    BinaryOp::Multiply => "mul",
                    _ => return Err(CodegenError::Unsupported("binary operator")),
                };

                let lhs = left.emit_llvm(ctx)?;
                let rhs = right.emit_llvm(ctx)?;
                let target = ctx.fresh_register();
                ctx.emit(&format!("{target} = {opcode} i32 {lhs}, {rhs}"));
                Ok(Operand::Register(target))
            }
            _ => Err(CodegenError::Unsupported("expression")),
        }
    }
}

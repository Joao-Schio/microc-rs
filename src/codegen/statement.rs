use crate::ast::statement::{Block, Statement, StatementKind};

use super::{CodegenError, ControlFlow, EmitLlvm, FunctionContext};

impl EmitLlvm for Statement {
    type Output = ControlFlow;

    fn emit_llvm(&self, ctx: &mut FunctionContext) -> Result<ControlFlow, CodegenError> {
        match &self.kind {
            StatementKind::Return {
                value: Some(expression),
            } => {
                let operand = expression.emit_llvm(ctx)?;
                ctx.emit(&format!("ret i32 {operand}"));
                Ok(ControlFlow::Terminated)
            }
            StatementKind::Empty => Ok(ControlFlow::Continues),
            StatementKind::Block(block) => block.emit_llvm(ctx),
            _ => Err(CodegenError::Unsupported("statement")),
        }
    }
}

impl EmitLlvm for Block {
    type Output = ControlFlow;

    fn emit_llvm(&self, ctx: &mut FunctionContext) -> Result<ControlFlow, CodegenError> {
        if !self.declarations.is_empty() {
            return Err(CodegenError::Unsupported("variable declarations"));
        }

        let mut flow = ControlFlow::Continues;
        for statement in &self.statements {
            if flow == ControlFlow::Terminated {
                return Err(CodegenError::UnreachableStatement {
                    line: statement.line(),
                });
            }

            flow = statement.emit_llvm(ctx)?;
        }

        Ok(flow)
    }
}

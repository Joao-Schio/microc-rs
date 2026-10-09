use crate::{
    codegen::{CodegenError, generate_ir},
    frontend::analyze_source,
};

fn lower(source: &[u8]) -> Result<String, CodegenError> {
    let program = analyze_source(source).expect("the test source must pass the frontend");
    generate_ir(&program)
}

#[test]
fn emits_integer_return() {
    let ir = lower(b"int main() { return 42; }").unwrap();

    assert_eq!(
        ir,
        concat!(
            "define i32 @main() {\n",
            "entry:\n",
            "  ret i32 42\n",
            "}\n"
        )
    );
}

#[test]
fn emits_nested_arithmetic_with_distinct_registers() {
    let ir = lower(b"int main() { return (3 + 4) * (8 - 2); }").unwrap();

    assert_eq!(
        ir,
        concat!(
            "define i32 @main() {\n",
            "entry:\n",
            "  %t0 = add i32 3, 4\n",
            "  %t1 = sub i32 8, 2\n",
            "  %t2 = mul i32 %t0, %t1\n",
            "  ret i32 %t2\n",
            "}\n"
        )
    );
}

#[test]
fn emits_unary_negation() {
    let ir = lower(b"int main() { return -(1 + 2); }").unwrap();

    assert!(ir.contains("  %t0 = add i32 1, 2\n"));
    assert!(ir.contains("  %t1 = sub i32 0, %t0\n"));
    assert!(ir.contains("  ret i32 %t1\n"));
}

#[test]
fn emits_empty_and_nested_blocks() {
    let ir = lower(b"int main() { ; { ; } { return 7; } }").unwrap();

    assert_eq!(ir, "define i32 @main() {\nentry:\n  ret i32 7\n}\n");
}

#[test]
fn rejects_additional_functions() {
    let result = lower(b"int answer() { return 42; } int main() { return 0; }");

    assert_eq!(
        result,
        Err(CodegenError::Unsupported("additional functions"))
    );
}

#[test]
fn rejects_variable_declarations() {
    let result = lower(b"int main() { int value; value = 7; return value; }");

    assert_eq!(
        result,
        Err(CodegenError::Unsupported("variable declarations"))
    );
}

#[test]
fn rejects_unsupported_statements() {
    let result = lower(b"int main() { print(42); return 0; }");

    assert_eq!(result, Err(CodegenError::Unsupported("statement")));
}

#[test]
fn rejects_unsupported_operators() {
    let result = lower(b"int main() { return 8 / 2; }");

    assert_eq!(result, Err(CodegenError::Unsupported("binary operator")));
}

#[test]
fn rejects_out_of_range_integer_literals() {
    let result = lower(b"int main() { return 2147483648; }");

    assert_eq!(result, Err(CodegenError::IntegerOutOfRange(2147483648)));
}

#[test]
fn detects_missing_return() {
    let result = lower(b"int main() { ; }");

    assert_eq!(result, Err(CodegenError::MissingReturn));
}

#[test]
fn rejects_code_after_an_unconditional_return() {
    let result = lower(b"int main() { return 1; return 2; }");

    assert_eq!(result, Err(CodegenError::UnreachableStatement { line: 1 }));
}

#[test]
fn formats_every_codegen_error() {
    let cases = [
        (
            CodegenError::Unsupported("array access"),
            "LLVM backend does not yet support array access",
        ),
        (
            CodegenError::IntegerOutOfRange(2147483648),
            "integer 2147483648 does not fit in i32",
        ),
        (
            CodegenError::MissingReturn,
            "function 'main' does not return a value",
        ),
        (
            CodegenError::UnreachableStatement { line: 12 },
            "LLVM backend does not yet handle unreachable statements at line 12",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
    }
}

#[test]
fn rejects_unsupported_expression_variants() {
    use crate::{
        ast::{Identifier, expression::Expression},
        codegen::{EmitLlvm, FunctionContext},
    };

    let name = || Identifier::new(b"value".to_vec(), 1);
    let unsupported = [
        Expression::Char(b'a'),
        Expression::Identifier(name()),
        Expression::ArrayAccess {
            array: name(),
            index: Box::new(Expression::Integer(0)),
        },
        Expression::Call {
            callee: name(),
            arguments: Vec::new(),
        },
    ];

    for expression in unsupported {
        let mut ctx = FunctionContext::new();
        assert!(matches!(
            expression.emit_llvm(&mut ctx),
            Err(CodegenError::Unsupported("expression"))
        ));
    }
}

#[test]
fn rejects_logical_not_until_boolean_lowering_exists() {
    use crate::{
        ast::expression::{Expression, UnaryOp},
        codegen::{EmitLlvm, FunctionContext},
    };

    let expression = Expression::Unary {
        op: UnaryOp::Not,
        expression: Box::new(Expression::Integer(1)),
    };

    let mut ctx = FunctionContext::new();
    assert!(matches!(
        expression.emit_llvm(&mut ctx),
        Err(CodegenError::Unsupported("unary operator"))
    ));
}

#[test]
fn rejects_return_without_a_value_at_the_codegen_boundary() {
    use crate::{
        ast::statement::{Statement, StatementKind},
        codegen::{EmitLlvm, FunctionContext},
    };

    // The semantic analyzer rejects this, but codegen must still fail
    // explicitly if invoked with an unvalidated AST.
    let statement = Statement::new(4, StatementKind::Return { value: None });

    let mut ctx = FunctionContext::new();
    assert_eq!(
        statement.emit_llvm(&mut ctx),
        Err(CodegenError::Unsupported("statement"))
    );
}

#[test]
fn rejects_out_of_range_integer_at_negative_boundary() {
    use crate::{
        ast::expression::Expression,
        codegen::{EmitLlvm, FunctionContext},
    };

    let mut ctx = FunctionContext::new();
    assert!(matches!(
        Expression::Integer(i64::MIN).emit_llvm(&mut ctx),
        Err(CodegenError::IntegerOutOfRange(i64::MIN))
    ));
}

#[test]
fn accepts_i32_integer_literal_boundaries() {
    use crate::{
        ast::expression::Expression,
        codegen::{EmitLlvm, FunctionContext},
    };

    for value in [i32::MIN, i32::MAX] {
        let mut ctx = FunctionContext::new();
        let operand = Expression::Integer(i64::from(value))
            .emit_llvm(&mut ctx)
            .expect("the i32 limits are valid integer operands");
        assert_eq!(operand.to_string(), value.to_string());
    }
}

#[test]
fn context_allocates_distinct_temporary_registers() {
    use crate::codegen::FunctionContext;

    let mut ctx = FunctionContext::new();
    assert_eq!(ctx.fresh_register(), "%t0");
    assert_eq!(ctx.fresh_register(), "%t1");
    ctx.emit("%t0 = add i32 1, 2");
    assert_eq!(ctx.into_instructions(), "  %t0 = add i32 1, 2\n");
}

#[test]
fn unsupported_binary_operator_does_not_emit_partial_ir() {
    use crate::{
        ast::expression::{BinaryOp, Expression},
        codegen::{EmitLlvm, FunctionContext},
    };

    let expression = Expression::Binary {
        left: Box::new(Expression::Integer(1)),
        op: BinaryOp::Divide,
        right: Box::new(Expression::Integer(0)),
    };
    let mut ctx = FunctionContext::new();

    assert!(matches!(
        expression.emit_llvm(&mut ctx),
        Err(CodegenError::Unsupported("binary operator"))
    ));
    assert_eq!(ctx.into_instructions(), "");
}

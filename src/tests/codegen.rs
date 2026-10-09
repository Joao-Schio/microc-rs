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

    assert_eq!(
        ir,
        "define i32 @main() {\nentry:\n  ret i32 7\n}\n"
    );
}

#[test]
fn rejects_additional_functions() {
    let result = lower(b"int answer() { return 42; } int main() { return 0; }");

    assert_eq!(result, Err(CodegenError::Unsupported("additional functions")));
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

    assert_eq!(
        result,
        Err(CodegenError::UnreachableStatement { line: 1 })
    );
}

use std::error::Error;

use crate::{
    ast::{
        Identifier,
        expression::{BinaryOp, UnaryOp},
        statement::Type,
    },
    frontend_error::FrontendError,
    lexer::LexerError,
    parser::ParserError,
    scanner::ScannerError,
    semantic::{ExprType, SemanticError},
    token::TokenType,
};

#[test]
fn semantic_error_displays_identifier_and_location() {
    let error = SemanticError::UndeclaredVariable(Identifier::new(b"value".to_vec(), 7));
    assert_eq!(error.to_string(), "undeclared variable 'value' at line 7");
}

#[test]
fn semantic_error_displays_expected_and_actual_types() {
    let error = SemanticError::TypeMismatch {
        expected: ExprType::Scalar(Type::Int),
        actual: ExprType::Scalar(Type::Char),
        line: 12,
    };

    assert_eq!(
        error.to_string(),
        "type mismatch at line 12: expected Scalar(Int), found Scalar(Char)"
    );
}

#[test]
fn semantic_error_displays_operator_diagnostics() {
    let binary = SemanticError::InvalidBinaryOperands {
        operator: BinaryOp::Add,
        left: ExprType::Scalar(Type::Int),
        right: ExprType::Scalar(Type::Char),
        line: 2,
    };
    let unary = SemanticError::InvalidUnaryOperand {
        operator: UnaryOp::Negate,
        actual: ExprType::Array(Type::Int),
        line: 3,
    };

    assert_eq!(
        binary.to_string(),
        "invalid operands for Add at line 2: Scalar(Int) and Scalar(Char)"
    );
    assert_eq!(
        unary.to_string(),
        "invalid operand for Negate at line 3: Array(Int)"
    );
}

#[test]
fn frontend_error_wraps_scanner_error_and_exposes_source() {
    let error: FrontendError = ScannerError::Io(std::io::Error::other("read failed")).into();

    assert!(matches!(&error, FrontendError::Scanner(_)));
    assert_eq!(error.to_string(), "scanner I/O error: read failed");
    assert_eq!(error.source().unwrap().to_string(), error.to_string());
}

#[test]
fn frontend_error_wraps_lexer_error_and_exposes_source() {
    let error: FrontendError = LexerError::UnterminatedString {
        line: 4,
        column: 10,
    }
    .into();

    assert!(matches!(&error, FrontendError::Lexer(_)));
    assert_eq!(
        error.to_string(),
        "unterminated string at line 4, column 10"
    );
    assert_eq!(error.source().unwrap().to_string(), error.to_string());
}

#[test]
fn frontend_error_wraps_parser_error_and_exposes_source() {
    let error: FrontendError = ParserError::UnexpectedToken {
        expected: "';'",
        found: TokenType::RBrace,
        line: 5,
    }
    .into();

    assert!(matches!(&error, FrontendError::Parser(_)));
    assert_eq!(error.to_string(), "expected ';', found RBrace at line 5");
    assert_eq!(error.source().unwrap().to_string(), error.to_string());
}

#[test]
fn frontend_error_wraps_semantic_error_and_exposes_source() {
    let error: FrontendError = SemanticError::MissingReturnValue { line: 9 }.into();

    assert!(matches!(&error, FrontendError::Semantic(_)));
    assert_eq!(error.to_string(), "missing return value at line 9");
    assert_eq!(error.source().unwrap().to_string(), error.to_string());
}

#[test]
fn frontend_error_supports_question_mark_propagation() {
    fn analyze() -> Result<(), FrontendError> {
        Err(SemanticError::MissingReturnValue { line: 3 })?;
        Ok(())
    }

    let error = analyze().expect_err("semantic error must propagate");
    assert!(matches!(
        error,
        FrontendError::Semantic(SemanticError::MissingReturnValue { line: 3 })
    ));
}

//! Stable, user-facing compiler diagnostics and source error chains.
use std::{
    error::Error,
    io,
};

use crate::{
    ast::{
        Identifier,
        expression::{BinaryOp, UnaryOp},
        statement::Type,
    },
    lexer::LexerError,
    parser::ParserError,
    scanner::ScannerError,
    semantic::{ExprType, SemanticError},
    token::TokenType,
};

fn identifier() -> Identifier {
    Identifier::new(b"ghost".to_vec(), 4)
}

#[test]
fn semantic_errors_explain_the_actual_failure_and_location() {
    let int = ExprType::Scalar(Type::Int);
    let character = ExprType::Scalar(Type::Char);

    let cases = [
        (
            SemanticError::UndeclaredVariable(identifier()),
            "undeclared variable 'ghost' at line 4",
        ),
        (
            SemanticError::DuplicateDeclaration(identifier()),
            "duplicate declaration of 'ghost' at line 4",
        ),
        (
            SemanticError::UndeclaredFunction(identifier()),
            "undeclared function 'ghost' at line 4",
        ),
        (
            SemanticError::NotCallable(identifier()),
            "'ghost' is not callable at line 4",
        ),
        (
            SemanticError::NotAVariable(identifier()),
            "'ghost' is not a variable at line 4",
        ),
        (
            SemanticError::NotAnArray(identifier()),
            "'ghost' is not an array at line 4",
        ),
        (
            SemanticError::NotAssignable(identifier()),
            "'ghost' is not assignable at line 4",
        ),
        (
            SemanticError::TypeMismatch {
                expected: int,
                actual: character,
                line: 4,
            },
            "type mismatch at line 4: expected Scalar(Int), found Scalar(Char)",
        ),
        (
            SemanticError::InvalidBinaryOperands {
                operator: BinaryOp::Add,
                left: int,
                right: character,
                line: 4,
            },
            "invalid operands for Add at line 4: Scalar(Int) and Scalar(Char)",
        ),
        (
            SemanticError::InvalidUnaryOperand {
                operator: UnaryOp::Not,
                actual: character,
                line: 4,
            },
            "invalid operand for Not at line 4: Scalar(Char)",
        ),
        (
            SemanticError::ArgumentCountMismatch {
                callee: identifier(),
                expected: 2,
                actual: 1,
            },
            "function 'ghost' at line 4 expects 2 arguments, found 1",
        ),
        (
            SemanticError::InvalidPrintType {
                actual: ExprType::Array(Type::Int),
                line: 4,
            },
            "invalid print type Array(Int) at line 4",
        ),
        (
            SemanticError::MissingReturnValue { line: 4 },
            "missing return value at line 4",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected, "{error:?}");
    }
}

#[test]
fn lexer_errors_explain_the_actual_failure_and_position() {
    let cases = [
        (
            LexerError::UnexpectedCharacter {
                character: b'@',
                line: 2,
                column: 8,
            },
            "unexpected character '@' at line 2, column 8",
        ),
        (
            LexerError::InvalidCharacterLiteral { line: 3, column: 7 },
            "invalid character literal at line 3, column 7",
        ),
        (
            LexerError::UnterminatedString { line: 4, column: 5 },
            "unterminated string at line 4, column 5",
        ),
        (
            LexerError::InvalidLogicalOperator {
                character: b'&',
                line: 5,
                column: 9,
            },
            "invalid logical operator '&' at line 5, column 9",
        ),
        (
            LexerError::IntegerOutOfRange {
                lexeme: "9223372036854775808".to_owned(),
                line: 6,
                column: 1,
            },
            "integer literal '9223372036854775808' is out of range at line 6, column 1",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected, "{error:?}");
        assert!(error.source().is_none());
    }
}

#[test]
fn parser_diagnostics_include_expected_token_and_line() {
    let cases = [
        (
            ParserError::UnexpectedToken {
                expected: "';'",
                found: TokenType::RBrace,
                line: 7,
            },
            "expected ';', found RBrace at line 7",
        ),
        (ParserError::ExpectedBlock, "expected a block"),
        (
            ParserError::InvalidMainReturnType { found: Type::Char },
            "main must return int, found Char",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
        assert!(error.source().is_none());
    }
}

#[test]
fn scanner_io_error_preserves_its_source_through_lexer() {
    let scanner = ScannerError::from(io::Error::other("read failed"));
    assert_eq!(scanner.to_string(), "scanner I/O error: read failed");
    assert_eq!(scanner.source().unwrap().to_string(), "read failed");

    let lexer = LexerError::from(scanner);
    assert_eq!(lexer.to_string(), "scanner I/O error: read failed");
    let scanner_source = lexer.source().expect("lexer keeps its scanner cause");
    assert_eq!(scanner_source.to_string(), "scanner I/O error: read failed");
    assert_eq!(
        scanner_source.source().unwrap().to_string(),
        "read failed"
    );
}

#[test]
fn diagnostics_handle_invalid_utf8_identifiers_without_panicking() {
    let error = SemanticError::UndeclaredVariable(Identifier::new(vec![0xff], 9));
    assert_eq!(error.to_string(), "undeclared variable '�' at line 9");
    assert!(error.source().is_none());
}

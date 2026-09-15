use crate::{
    lexer::{tokenize, LexerError},
    token::{Token, TokenType},
};

use super::helpers::make_lexer;

fn assert_token(token: &Token, expected_type: TokenType, expected_lexeme: &[u8]) {
    assert_eq!(token.get_tok_type(), &expected_type);
    assert_eq!(token.get_lexema(), expected_lexeme);
}

#[test]
fn empty_source_returns_only_eof() {
    let mut lexer = make_lexer("");

    let tokens = tokenize(&mut lexer).expect("tokenization should succeed");

    assert_eq!(tokens.len(), 1);
    assert_token(&tokens[0], TokenType::Eof, b"");
}

#[test]
fn collects_tokens_until_eof() {
    let mut lexer = make_lexer("value + 123");

    let tokens = tokenize(&mut lexer).expect("tokenization should succeed");

    assert_eq!(tokens.len(), 4);

    assert_token(&tokens[0], TokenType::Id, b"value");
    assert_token(&tokens[1], TokenType::Plus, b"+");
    assert_token(&tokens[2], TokenType::IntegerConst(123), b"123");
    assert_token(&tokens[3], TokenType::Eof, b"");
}

#[test]
fn returns_error_when_lexical_analysis_fails() {
    let mut lexer = make_lexer("@");

    let result = tokenize(&mut lexer);

    assert!(matches!(
        result,
        Err(LexerError::UnexpectedCharacter {
            character: b'@',
            ..
        })
    ));
}

#[test]
fn does_not_return_partial_tokens_after_error() {
    let mut lexer = make_lexer("value + @");

    let result = tokenize(&mut lexer);

    assert!(matches!(
        result,
        Err(LexerError::UnexpectedCharacter {
            character: b'@',
            ..
        })
    ));
}
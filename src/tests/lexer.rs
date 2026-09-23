use std::collections::HashMap;

use crate::{lexer::LexerError, token::TokenType};

use super::helpers::{
    assert_lexer_error, assert_token, make_failing_lexer, make_lexer,
    make_lexer_with_reserved_words,
};

fn assert_single_char_token(input: &str, expected_type: TokenType) {
    let mut lexer = make_lexer(input);
    assert_token(&mut lexer, expected_type, input);
}

#[test]
fn recognizes_plus() {
    assert_single_char_token("+", TokenType::Plus);
}

#[test]
fn recognizes_minus() {
    assert_single_char_token("-", TokenType::Minus);
}

#[test]
fn recognizes_simple_assignment() {
    let mut lexer = make_lexer("=");
    assert_token(&mut lexer, TokenType::Assign, "=");
}

#[test]
fn recognizes_equals() {
    let mut lexer = make_lexer("==");
    assert_token(&mut lexer, TokenType::Eq, "==");
}

#[test]
fn recognizes_complex_assignment() {
    let mut lexer = make_lexer("=123");
    assert_token(&mut lexer, TokenType::Assign, "=");
}

#[test]
fn recognizes_greater_than() {
    let mut lexer = make_lexer(">");
    assert_token(&mut lexer, TokenType::Gt, ">");
}

#[test]
fn recognizes_greater_or_equal() {
    let mut lexer = make_lexer(">=");
    assert_token(&mut lexer, TokenType::Geq, ">=");
}

#[test]
fn equality_consumes_equal() {
    let mut lexer = make_lexer("==+");
    assert_token(&mut lexer, TokenType::Eq, "==");
    assert_token(&mut lexer, TokenType::Plus, "+");
}

#[test]
fn geq_consumes_equals() {
    let mut lexer = make_lexer(">=+");
    assert_token(&mut lexer, TokenType::Geq, ">=");
    assert_token(&mut lexer, TokenType::Plus, "+");
}

#[test]
fn recognizes_less_than() {
    let mut lexer = make_lexer("<");
    assert_token(&mut lexer, TokenType::Lt, "<");
}

#[test]
fn recognizes_less_or_equal() {
    let mut lexer = make_lexer("<=");
    assert_token(&mut lexer, TokenType::Leq, "<=");
}

#[test]
fn leq_consumes_equals() {
    let mut lexer = make_lexer("<=+");
    assert_token(&mut lexer, TokenType::Leq, "<=");
    assert_token(&mut lexer, TokenType::Plus, "+");
}

#[test]
fn less_than_does_not_consume_next() {
    let mut lexer = make_lexer("<+");
    assert_token(&mut lexer, TokenType::Lt, "<");
    assert_token(&mut lexer, TokenType::Plus, "+");
}

#[test]
fn recognizes_logical_and() {
    let mut lexer = make_lexer("&&");
    assert_token(&mut lexer, TokenType::And, "&&");
}

#[test]
fn logical_and_consumes_both_characters() {
    let mut lexer = make_lexer("&&+");
    assert_token(&mut lexer, TokenType::And, "&&");
    assert_token(&mut lexer, TokenType::Plus, "+");
}

#[test]
fn single_ampersand_is_error() {
    let mut lexer = make_lexer("&");
    let error = assert_lexer_error(&mut lexer);

    assert!(matches!(
        error,
        LexerError::InvalidLogicalOperator {
            character: b'&',
            ..
        }
    ));
}

#[test]
fn invalid_ampersand_does_not_consume_next() {
    let mut lexer = make_lexer("&+");
    let error = assert_lexer_error(&mut lexer);

    assert!(matches!(
        error,
        LexerError::InvalidLogicalOperator {
            character: b'&',
            ..
        }
    ));
    assert_token(&mut lexer, TokenType::Plus, "+");
}

#[test]
fn recognizes_logical_or() {
    let mut lexer = make_lexer("||");
    assert_token(&mut lexer, TokenType::Or, "||");
}

#[test]
fn logical_or_consumes_both_characters() {
    let mut lexer = make_lexer("||+");
    assert_token(&mut lexer, TokenType::Or, "||");
    assert_token(&mut lexer, TokenType::Plus, "+");
}

#[test]
fn single_pipe_is_error() {
    let mut lexer = make_lexer("|");
    let error = assert_lexer_error(&mut lexer);

    assert!(matches!(
        error,
        LexerError::InvalidLogicalOperator {
            character: b'|',
            ..
        }
    ));
}

#[test]
fn invalid_pipe_does_not_consume_next() {
    let mut lexer = make_lexer("|+");
    let error = assert_lexer_error(&mut lexer);

    assert!(matches!(
        error,
        LexerError::InvalidLogicalOperator {
            character: b'|',
            ..
        }
    ));
    assert_token(&mut lexer, TokenType::Plus, "+");
}

#[test]
fn recognizes_not() {
    let mut lexer = make_lexer("!");
    assert_token(&mut lexer, TokenType::Not, "!");
}

#[test]
fn recognizes_not_equal() {
    let mut lexer = make_lexer("!=");
    assert_token(&mut lexer, TokenType::Neq, "!=");
}

#[test]
fn recognizes_char_const() {
    let mut lexer = make_lexer("'a'");
    assert_token(&mut lexer, TokenType::CharConst(b'a'), "a");
}

#[test]
fn recognizes_symbol_char_const() {
    let mut lexer = make_lexer("'+'");
    assert_token(&mut lexer, TokenType::CharConst(b'+'), "+");
}

#[test]
fn char_const_consumes_closing_quote() {
    let mut lexer = make_lexer("'a'+");
    assert_token(&mut lexer, TokenType::CharConst(b'a'), "a");
    assert_token(&mut lexer, TokenType::Plus, "+");
}

#[test]
fn unterminated_char_const_is_error() {
    let mut lexer = make_lexer("'a");
    let error = assert_lexer_error(&mut lexer);

    assert!(matches!(error, LexerError::InvalidCharacterLiteral { .. }));
}

#[test]
fn empty_char_const_is_error() {
    let mut lexer = make_lexer("''");
    let error = assert_lexer_error(&mut lexer);

    assert!(matches!(error, LexerError::InvalidCharacterLiteral { .. }));
}

#[test]
fn multiple_character_char_const_is_error() {
    let mut lexer = make_lexer("'ab'");
    let error = assert_lexer_error(&mut lexer);

    assert!(matches!(error, LexerError::InvalidCharacterLiteral { .. }));
}

#[test]
fn newline_in_char_const_is_error() {
    let mut lexer = make_lexer("'\n'");
    let error = assert_lexer_error(&mut lexer);

    assert!(matches!(error, LexerError::InvalidCharacterLiteral { .. }));
}

#[test]
fn recognizes_string_const() {
    let mut lexer = make_lexer("\"hello\"");
    assert_token(
        &mut lexer,
        TokenType::StringConst(b"hello".to_vec()),
        "hello",
    );
}

#[test]
fn recognizes_empty_string_const() {
    let mut lexer = make_lexer("\"\"");
    assert_token(&mut lexer, TokenType::StringConst(Vec::new()), "");
}

#[test]
fn recognizes_string_const_with_symbols() {
    let mut lexer = make_lexer("\"123 !@#$%\"");
    assert_token(
        &mut lexer,
        TokenType::StringConst(b"123 !@#$%".to_vec()),
        "123 !@#$%",
    );
}

#[test]
fn string_const_consumes_closing_quote() {
    let mut lexer = make_lexer("\"hello\"+");
    assert_token(
        &mut lexer,
        TokenType::StringConst(b"hello".to_vec()),
        "hello",
    );
    assert_token(&mut lexer, TokenType::Plus, "+");
}

#[test]
fn unterminated_string_const_is_error() {
    let mut lexer = make_lexer("\"hello");
    let error = assert_lexer_error(&mut lexer);

    assert!(matches!(error, LexerError::UnterminatedString { .. }));
}

#[test]
fn newline_in_string_const_is_error() {
    let mut lexer = make_lexer("\"hello\nworld\"");
    let error = assert_lexer_error(&mut lexer);

    assert!(matches!(error, LexerError::UnterminatedString { .. }));
}

#[test]
fn recognizes_comma() {
    assert_single_char_token(",", TokenType::Comma);
}

#[test]
fn recognizes_semicolon() {
    assert_single_char_token(";", TokenType::SemiColon);
}

#[test]
fn recognizes_left_parenthesis() {
    assert_single_char_token("(", TokenType::Lparen);
}

#[test]
fn recognizes_right_parenthesis() {
    assert_single_char_token(")", TokenType::Rparen);
}

#[test]
fn recognizes_left_brace() {
    assert_single_char_token("{", TokenType::LBrace);
}

#[test]
fn recognizes_right_brace() {
    assert_single_char_token("}", TokenType::RBrace);
}

#[test]
fn recognizes_left_bracket() {
    assert_single_char_token("[", TokenType::LBracket);
}

#[test]
fn recognizes_right_bracket() {
    assert_single_char_token("]", TokenType::RBracket);
}

#[test]
fn recognizes_mul() {
    assert_single_char_token("*", TokenType::Mul);
}

#[test]
fn recognizes_mod() {
    assert_single_char_token("%", TokenType::Mod);
}

#[test]
fn recognizes_identifier() {
    let mut lexer = make_lexer("value");
    assert_token(&mut lexer, TokenType::Id(b"value".to_vec()), "value");
}

#[test]
fn recognizes_identifier_with_uppercase_digits_and_underscore() {
    let mut lexer = make_lexer("Value_123");
    assert_token(
        &mut lexer,
        TokenType::Id(b"Value_123".to_vec()),
        "Value_123",
    );
}

#[test]
fn recognizes_identifier_starting_with_underscore() {
    let mut lexer = make_lexer("_value");
    assert_token(&mut lexer, TokenType::Id(b"_value".to_vec()), "_value");
}

#[test]
fn recognizes_injected_reserved_word() {
    let reserved_words = HashMap::from([("if", TokenType::If)]);
    let mut lexer = make_lexer_with_reserved_words("if", reserved_words);
    assert_token(&mut lexer, TokenType::If, "if");
}

#[test]
fn reserved_word_match_is_exact() {
    let reserved_words = HashMap::from([("if", TokenType::If)]);
    let mut lexer = make_lexer_with_reserved_words("ifx", reserved_words);
    assert_token(&mut lexer, TokenType::Id(b"ifx".to_vec()), "ifx");
}

#[test]
fn identifier_does_not_consume_following_token() {
    let mut lexer = make_lexer("value+");
    assert_token(&mut lexer, TokenType::Id(b"value".to_vec()), "value");
    assert_token(&mut lexer, TokenType::Plus, "+");
}

#[test]
fn unexpected_character_is_error() {
    let mut lexer = make_lexer("@");
    let error = assert_lexer_error(&mut lexer);

    assert!(matches!(
        error,
        LexerError::UnexpectedCharacter {
            character: b'@',
            ..
        }
    ));
}

#[test]
fn empty_input_returns_eof() {
    let mut lexer = make_lexer("");
    assert_token(&mut lexer, TokenType::Eof, "");
}

#[test]
fn eof_follows_last_token() {
    let mut lexer = make_lexer("+");
    assert_token(&mut lexer, TokenType::Plus, "+");
    assert_token(&mut lexer, TokenType::Eof, "");
}

#[test]
fn repeated_eof_is_stable() {
    let mut lexer = make_lexer("");
    assert_token(&mut lexer, TokenType::Eof, "");
    assert_token(&mut lexer, TokenType::Eof, "");
}

#[test]
fn scanner_error_is_propagated() {
    let mut lexer = make_failing_lexer();
    let error = assert_lexer_error(&mut lexer);

    assert!(matches!(error, LexerError::Scanner(_)));
}

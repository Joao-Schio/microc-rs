use crate::{lexer::LexerError, token::TokenType};

use super::helpers::{assert_lexer_error, assert_token, make_lexer};

#[test]
fn recognizes_zero() {
    let mut lexer = make_lexer("0");
    assert_token(&mut lexer, TokenType::IntegerConst(0), "0");
}

#[test]
fn recognizes_nine() {
    let mut lexer = make_lexer("9");
    assert_token(&mut lexer, TokenType::IntegerConst(9), "9");
}

#[test]
fn recognizes_multi_digit_integer() {
    let mut lexer = make_lexer("12341235");
    assert_token(&mut lexer, TokenType::IntegerConst(12_341_235), "12341235");
}

#[test]
fn integer_lexeme_is_reconstructed_from_value() {
    let mut lexer = make_lexer("00123");
    assert_token(&mut lexer, TokenType::IntegerConst(123), "123");
}

#[test]
fn integer_does_not_consume_following_operator() {
    let mut lexer = make_lexer("123+");
    assert_token(&mut lexer, TokenType::IntegerConst(123), "123");
    assert_token(&mut lexer, TokenType::Plus, "+");
}

#[test]
fn integer_does_not_consume_following_identifier() {
    let mut lexer = make_lexer("123abc");
    assert_token(&mut lexer, TokenType::IntegerConst(123), "123");
    assert_token(&mut lexer, TokenType::Id(b"abc".to_vec()), "abc");
}

#[test]
fn scientific_notation_is_not_an_integer_literal() {
    let mut lexer = make_lexer("12E2");
    assert_token(&mut lexer, TokenType::IntegerConst(12), "12");
    assert_token(&mut lexer, TokenType::Id(b"E2".to_vec()), "E2");
}

#[test]
fn negative_value_is_minus_followed_by_integer() {
    let mut lexer = make_lexer("-123");
    assert_token(&mut lexer, TokenType::Minus, "-");
    assert_token(&mut lexer, TokenType::IntegerConst(123), "123");
}

#[test]
fn integer_out_of_range_is_error() {
    let mut lexer = make_lexer("9223372036854775808");
    let error = assert_lexer_error(&mut lexer);

    assert!(matches!(error, LexerError::IntegerOutOfRange { .. }));
}

use std::collections::HashMap;

use crate::token::TokenType;

use super::helpers::{assert_token, make_lexer_with_reserved_words};

fn assert_reserved_word(input: &'static str, expected_type: TokenType) {
    let reserved_words = HashMap::from([(input, expected_type.clone())]);
    let mut lexer = make_lexer_with_reserved_words(input, reserved_words);
    assert_token(&mut lexer, expected_type, input);
}

#[test]
fn recognizes_main_reserved_word() {
    assert_reserved_word("main", TokenType::Main);
}

#[test]
fn recognizes_if_reserved_word() {
    assert_reserved_word("if", TokenType::If);
}

#[test]
fn recognizes_else_reserved_word() {
    assert_reserved_word("else", TokenType::Else);
}

#[test]
fn recognizes_for_reserved_word() {
    assert_reserved_word("for", TokenType::For);
}

#[test]
fn recognizes_return_reserved_word() {
    assert_reserved_word("return", TokenType::Return);
}

#[test]
fn recognizes_int_reserved_word() {
    assert_reserved_word("int", TokenType::Int);
}

#[test]
fn recognizes_char_reserved_word() {
    assert_reserved_word("char", TokenType::Char);
}

#[test]
fn recognizes_print_reserved_word() {
    assert_reserved_word("print", TokenType::Print);
}

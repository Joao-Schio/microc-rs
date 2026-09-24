use crate::token::TokenType;

use super::helpers::assert_token;

lexer_contract!(default_lexer, crate::tests::helpers::make_lexer, {
    fn assert_reserved_word(input: &str, expected_type: TokenType) {
        let mut lexer = make_lexer(input);
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
});

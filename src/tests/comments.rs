use crate::token::TokenType;

use super::helpers::assert_token;

lexer_contract!(default_lexer, crate::tests::helpers::make_lexer, {
    #[test]
    fn recognizes_div() {
        let mut lexer = make_lexer("/");
        assert_token(&mut lexer, TokenType::Div, "/");
    }

    #[test]
    fn division_does_not_consume_following_token() {
        let mut lexer = make_lexer("/+");
        assert_token(&mut lexer, TokenType::Div, "/");
        assert_token(&mut lexer, TokenType::Plus, "+");
    }

    #[test]
    fn skips_empty_line_comment() {
        let mut lexer = make_lexer("//\n+");
        assert_token(&mut lexer, TokenType::Plus, "+");
    }

    #[test]
    fn skips_line_comment_with_content() {
        let mut lexer = make_lexer("//abc\n+");
        assert_token(&mut lexer, TokenType::Plus, "+");
    }

    #[test]
    fn skips_comment_between_tokens() {
        let mut lexer = make_lexer("+//abc\n-");
        assert_token(&mut lexer, TokenType::Plus, "+");
        assert_token(&mut lexer, TokenType::Minus, "-");
    }

    #[test]
    fn skips_consecutive_comments() {
        let mut lexer = make_lexer("//a\n//b\n+");
        assert_token(&mut lexer, TokenType::Plus, "+");
    }
});

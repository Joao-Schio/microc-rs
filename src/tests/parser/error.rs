use crate::token::{Token, TokenType};

parser_contract!(
    default_parser,
    crate::parser::ExprParser,
    crate::parser::StatementParser,
    {
        #[test]
        fn parser_error_message_includes_source_line() {
            let tokens = vec![Token::new(TokenType::Plus, 7)];
            let mut parser = make_parser(tokens);

            let error = parser
                .parse_expression()
                .expect_err("plus token should not parse as an expression");

            assert_eq!(
                error.to_string(),
                "expected expression, found Plus at line 7"
            );
        }
    }
);

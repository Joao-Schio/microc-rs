use crate::{
    parser::Parser,
    token::{Token, TokenType},
};

#[test]
fn parser_error_message_includes_source_line() {
    let tokens = vec![Token::new(TokenType::Plus, 7, b"+".to_vec())];
    let mut parser = Parser::new(tokens);

    let error = parser
        .parse_expression()
        .expect_err("plus token should not parse as an expression");

    assert_eq!(
        error.to_string(),
        "expected expression, found Plus at line 7"
    );
}

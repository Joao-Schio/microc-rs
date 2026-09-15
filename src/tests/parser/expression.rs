use crate::{parser::{Parser, ast::expression::Expression}, token::{Token, TokenType}};

#[test]
fn parses_integer_expression() {
    let tokens = vec![
        Token::new(
            TokenType::IntegerConst(42),
            1,
            b"42".to_vec(),
        ),
        Token::new(
            TokenType::Eof,
            1,
            vec![],
        ),
    ];

    let mut parser = Parser::new(&tokens);

    let expression = parser
        .parse_expression()
        .expect("integer expression should parse");

    assert_eq!(
        expression,
        Expression::Integer(42)
    );
}
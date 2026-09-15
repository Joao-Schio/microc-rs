use crate::{
    ast::expression::Expression,
    parser::{Parser, ParserError},
    token::{Token, TokenType},
};

#[test]
fn parses_integer_expression() {
    let tokens = vec![
        Token::new(TokenType::IntegerConst(42), 1, b"42".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    let mut parser = Parser::new(&tokens);

    let expression = parser
        .parse_expression()
        .expect("integer expression should parse");

    assert_eq!(expression, Expression::Integer(42));
}

#[test]
fn non_expression_token_returns_parser_error() {
    let tokens = vec![Token::new(TokenType::Plus, 1, b"+".to_vec())];
    let mut parser = Parser::new(&tokens);

    let result = parser.parse_expression();

    assert_eq!(
        result,
        Err(ParserError::UnexpectedToken {
            expected: "expression",
            found: TokenType::Plus,
        })
    );
}

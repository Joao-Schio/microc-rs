use crate::{
    ast::expression::{Expression, UnaryOp},
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

#[test]
fn parses_identifier_expression() {
    let tokens = vec![
        Token::new(TokenType::Id, 1, b"value".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    let mut parser = Parser::new(&tokens);

    let expression = parser
        .parse_expression()
        .expect("identifier expression should parse");

    assert_eq!(expression, Expression::Identifier(b"value".to_vec()));
}

#[test]
fn parses_char_expression() {
    let tokens = vec![
        Token::new(TokenType::CharConst, 1, b"a".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    let mut parser = Parser::new(&tokens);

    let expression = parser
        .parse_expression()
        .expect("character expression should parse");

    assert_eq!(expression, Expression::Char(b'a'));
}

#[test]
fn parses_parenthesized_expression() {
    let tokens = vec![
        Token::new(TokenType::Lparen, 1, b"(".to_vec()),
        Token::new(TokenType::IntegerConst(42), 1, b"42".to_vec()),
        Token::new(TokenType::Rparen, 1, b")".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    let mut parser = Parser::new(&tokens);

    let expression = parser
        .parse_expression()
        .expect("parenthesized expression should parse");

    assert_eq!(expression, Expression::Integer(42));
}

#[test]
fn parenthesized_expression_requires_closing_parenthesis() {
    let tokens = vec![
        Token::new(TokenType::Lparen, 1, b"(".to_vec()),
        Token::new(TokenType::IntegerConst(42), 1, b"42".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    let mut parser = Parser::new(&tokens);

    assert_eq!(
        parser.parse_expression(),
        Err(ParserError::UnexpectedToken {
            expected: "')'",
            found: TokenType::Eof,
        })
    );
}

#[test]
fn parses_logical_not_expression() {
    let tokens = vec![
        Token::new(TokenType::Not, 1, b"!".to_vec()),
        Token::new(TokenType::IntegerConst(42), 1, b"42".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    let mut parser = Parser::new(&tokens);

    let expression = parser
        .parse_expression()
        .expect("logical not expression should parse");

    assert_eq!(
        expression,
        Expression::Unary {
            op: UnaryOp::Not,
            expression: Box::new(Expression::Integer(42)),
        }
    );
}

#[test]
fn parses_negative_number() {
    let tokens = vec![
        Token::new(TokenType::Minus, 1, b"-".to_vec()),
        Token::new(TokenType::IntegerConst(42), 1, b"42".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    let mut parser = Parser::new(&tokens);

    let expression = parser
        .parse_expression()
        .expect("negative number expression should parse");

    assert_eq!(
        expression,
        Expression::Unary {
            op: UnaryOp::Negate,
            expression: Box::new(Expression::Integer(42)),
        }
    );
}

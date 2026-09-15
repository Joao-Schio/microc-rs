use crate::{
    ast::expression::{BinaryOp, Expression},
    parser::{Parser, ParserError},
    token::{Token, TokenType},
};

#[test]
fn parses_array_access_expression() {
    let tokens = vec![
        Token::new(TokenType::Id, 1, b"values".to_vec()),
        Token::new(TokenType::LBracket, 1, b"[".to_vec()),
        Token::new(TokenType::IntegerConst(2), 1, b"2".to_vec()),
        Token::new(TokenType::Plus, 1, b"+".to_vec()),
        Token::new(TokenType::IntegerConst(3), 1, b"3".to_vec()),
        Token::new(TokenType::RBracket, 1, b"]".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    let mut parser = Parser::new(&tokens);

    let expression = parser
        .parse_expression()
        .expect("array access expression should parse");

    assert_eq!(
        expression,
        Expression::ArrayAccess {
            array: b"values".to_vec(),
            index: Box::new(Expression::Binary {
                left: Box::new(Expression::Integer(2)),
                op: BinaryOp::Add,
                right: Box::new(Expression::Integer(3)),
            }),
        }
    );
}

#[test]
fn array_access_requires_closing_bracket() {
    let tokens = vec![
        Token::new(TokenType::Id, 1, b"values".to_vec()),
        Token::new(TokenType::LBracket, 1, b"[".to_vec()),
        Token::new(TokenType::IntegerConst(2), 1, b"2".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    let mut parser = Parser::new(&tokens);

    assert_eq!(
        parser.parse_expression(),
        Err(ParserError::UnexpectedToken {
            expected: "']'",
            found: TokenType::Eof,
            line: 1
        })
    );
}

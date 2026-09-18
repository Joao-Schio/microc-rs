use crate::{
    ast::{
        expression::Expression,
        statement::{AssignmentTarget, Statement},
    },
    parser::Parser,
    token::{Token, TokenType},
};

#[test]
fn parses_identifier_assignment() {
    let tokens = vec![
        Token::new(TokenType::Id, 1, b"x".to_vec()),
        Token::new(TokenType::Assign, 1, b"=".to_vec()),
        Token::new(TokenType::IntegerConst(42), 1, b"42".to_vec()),
        Token::new(TokenType::SemiColon, 1, b";".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    let mut parser = Parser::new(&tokens);

    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::Assignment {
            target: AssignmentTarget::Identifier(b"x".to_vec()),
            value: Expression::Integer(42),
        })
    );
}

#[test]
fn parses_consecutive_assignments() {
    let tokens = vec![
        // x = 1;
        Token::new(TokenType::Id, 1, b"x".to_vec()),
        Token::new(TokenType::Assign, 1, b"=".to_vec()),
        Token::new(TokenType::IntegerConst(1), 1, b"1".to_vec()),
        Token::new(TokenType::SemiColon, 1, b";".to_vec()),
        // y = 2;
        Token::new(TokenType::Id, 2, b"y".to_vec()),
        Token::new(TokenType::Assign, 2, b"=".to_vec()),
        Token::new(TokenType::IntegerConst(2), 2, b"2".to_vec()),
        Token::new(TokenType::SemiColon, 2, b";".to_vec()),
        Token::new(TokenType::Eof, 2, vec![]),
    ];

    let mut parser = Parser::new(&tokens);

    assert!(parser.parse_statement().is_ok());
    assert!(parser.parse_statement().is_ok());
}

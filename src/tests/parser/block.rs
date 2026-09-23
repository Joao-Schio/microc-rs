use crate::{
    ast::{
        expression::Expression,
        statement::{AssignmentTarget, Block, Statement},
    },
    parser::Parser,
    token::{Token, TokenType},
};

#[test]
fn parses_empty_block() {
    let tokens = vec![
        Token::new(TokenType::LBrace, 1),
        Token::new(TokenType::RBrace, 1),
        Token::new(TokenType::Eof, 1),
    ];

    let mut parser = Parser::new(tokens);

    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::Block(Block {
            declarations: vec![],
            statements: vec![],
        }))
    );
}

#[test]
fn parses_block_with_multiple_statements() {
    let tokens = vec![
        Token::new(TokenType::LBrace, 1),
        Token::new(TokenType::Id(b"x".to_vec()), 2),
        Token::new(TokenType::Assign, 2),
        Token::new(TokenType::IntegerConst(10), 2),
        Token::new(TokenType::SemiColon, 2),
        Token::new(TokenType::Return, 3),
        Token::new(TokenType::IntegerConst(0), 3),
        Token::new(TokenType::SemiColon, 3),
        Token::new(TokenType::RBrace, 4),
        Token::new(TokenType::Eof, 4),
    ];

    let mut parser = Parser::new(tokens);

    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::Block(Block {
            declarations: vec![],
            statements: vec![
                Statement::Assignment {
                    target: AssignmentTarget::Identifier(b"x".to_vec()),
                    value: Expression::Integer(10),
                },
                Statement::Return {
                    value: Some(Expression::Integer(0)),
                },
            ],
        }))
    );
}

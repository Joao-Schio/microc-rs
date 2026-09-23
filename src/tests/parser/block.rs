use crate::{
    ast::statement::{Block, Statement},
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

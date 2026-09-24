use crate::{
    ast::{
        program::{FunctionDefinition, Program},
        statement::Block,
        statement::Type,
    },
    parser::Parser,
    token::{Token, TokenType},
};

#[test]
fn parses_empty_main_function() {
    let tokens = vec![
        Token::new(TokenType::Int, 1),
        Token::new(TokenType::Id(b"main".to_vec()), 1),
        Token::new(TokenType::Lparen, 1),
        Token::new(TokenType::Rparen, 1),
        Token::new(TokenType::LBrace, 1),
        Token::new(TokenType::RBrace, 1),
        Token::new(TokenType::Eof, 1),
    ];

    let mut parser = Parser::new(tokens);

    assert_eq!(
        parser.parse_program(),
        Ok(Program {
            functions: vec![FunctionDefinition {
                return_type: Type::Int,
                name: b"main".to_vec(),
                parameters: vec![],
                body: Block {
                    declarations: vec![],
                    statements: vec![],
                },
            }],
        })
    );
}

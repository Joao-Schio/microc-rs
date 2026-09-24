use crate::{
    ast::{
        program::{GenericFunction, MainFunction, Program},
        statement::{Block, Type},
    },
    parser::{Parser, ParserError},
    token::{Token, TokenType},
};

fn empty_block() -> Block {
    Block {
        declarations: vec![],
        statements: vec![],
    }
}

#[test]
fn parses_empty_main_function() {
    let tokens = vec![
        Token::new(TokenType::Int, 1),
        Token::new(TokenType::Main, 1),
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
            functions: vec![],
            main: MainFunction {
                body: empty_block(),
            },
        })
    );
}

#[test]
fn parses_generic_function_before_main() {
    let tokens = vec![
        Token::new(TokenType::Int, 1),
        Token::new(TokenType::Id(b"helper".to_vec()), 1),
        Token::new(TokenType::Lparen, 1),
        Token::new(TokenType::Rparen, 1),
        Token::new(TokenType::LBrace, 1),
        Token::new(TokenType::RBrace, 1),
        Token::new(TokenType::Int, 2),
        Token::new(TokenType::Main, 2),
        Token::new(TokenType::Lparen, 2),
        Token::new(TokenType::Rparen, 2),
        Token::new(TokenType::LBrace, 2),
        Token::new(TokenType::RBrace, 2),
        Token::new(TokenType::Eof, 2),
    ];

    let mut parser = Parser::new(tokens);

    assert_eq!(
        parser.parse_program(),
        Ok(Program {
            functions: vec![GenericFunction {
                return_type: Type::Int,
                name: b"helper".to_vec(),
                parameters: vec![],
                body: empty_block(),
            }],
            main: MainFunction {
                body: empty_block(),
            },
        })
    );
}

#[test]
fn rejects_non_int_main() {
    let tokens = vec![
        Token::new(TokenType::Char, 1),
        Token::new(TokenType::Main, 1),
        Token::new(TokenType::Lparen, 1),
        Token::new(TokenType::Rparen, 1),
        Token::new(TokenType::LBrace, 1),
        Token::new(TokenType::RBrace, 1),
        Token::new(TokenType::Eof, 1),
    ];

    let mut parser = Parser::new(tokens);

    assert_eq!(
        parser.parse_program(),
        Err(ParserError::InvalidMainReturnType { found: Type::Char })
    );
}

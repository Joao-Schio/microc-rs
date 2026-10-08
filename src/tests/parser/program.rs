use crate::{
    ast::{
        Identifier,
        program::{GenericFunction, MainFunction, Parameter, Program},
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
                name: identifier!(b"helper"),
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

#[test]
fn parses_generic_function_with_multiple_arguments() {
    let tokens = vec![
        Token::new(TokenType::Char, 1),
        Token::new(TokenType::Id(b"function".to_vec()), 1),
        Token::new(TokenType::Lparen, 1),
        Token::new(TokenType::Int, 1),
        Token::new(TokenType::Id(b"arg1".to_vec()), 1),
        Token::new(TokenType::Comma, 1),
        Token::new(TokenType::Int, 1),
        Token::new(TokenType::Id(b"arg2".to_vec()), 1),
        Token::new(TokenType::Comma, 1),
        Token::new(TokenType::Char, 1),
        Token::new(TokenType::Id(b"arg3".to_vec()), 1),
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
    let generic_function = parser
        .parse_program()
        .expect("function should parse")
        .functions
        .into_iter()
        .next()
        .expect("expected a generic function");

    assert_eq!(
        generic_function.name,
        Identifier {
            name: b"function".to_vec(),
            line: 1
        }
    );
    assert_eq!(
        generic_function.parameters,
        vec![
            Parameter {
                data_type: Type::Int,
                name: Identifier {
                    name: b"arg1".to_vec(),
                    line: 1
                }
            },
            Parameter {
                data_type: Type::Int,
                name: Identifier {
                    name: b"arg2".to_vec(),
                    line: 1
                }
            },
            Parameter {
                data_type: Type::Char,
                name: Identifier {
                    name: b"arg3".to_vec(),
                    line: 1
                }
            }
        ]
    )
}

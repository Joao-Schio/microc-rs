use crate::{
    ast::{
        expression::Expression,
        statement::{AssignmentTarget, Block, DataType, Statement, VariableDeclaration},
    },
    parser::{Parser, ParserError},
    token::{Token, TokenType},
};

#[test]
fn parses_int_scalar_declaration() {
    let tokens = vec![
        Token::new(TokenType::LBrace, 1),
        Token::new(TokenType::Int, 1),
        Token::new(TokenType::Id(b"count".to_vec()), 1),
        Token::new(TokenType::SemiColon, 1),
        Token::new(TokenType::RBrace, 1),
        Token::new(TokenType::Eof, 1),
    ];

    let mut parser = Parser::new(tokens);

    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::Block(Block {
            declarations: vec![VariableDeclaration::Scalar {
                data_type: DataType::Int,
                name: b"count".to_vec(),
            }],
            statements: vec![],
        }))
    );
}

#[test]
fn parses_char_scalar_declaration() {
    let tokens = vec![
        Token::new(TokenType::LBrace, 1),
        Token::new(TokenType::Char, 1),
        Token::new(TokenType::Id(b"letter".to_vec()), 1),
        Token::new(TokenType::SemiColon, 1),
        Token::new(TokenType::RBrace, 1),
        Token::new(TokenType::Eof, 1),
    ];

    let mut parser = Parser::new(tokens);

    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::Block(Block {
            declarations: vec![VariableDeclaration::Scalar {
                data_type: DataType::Char,
                name: b"letter".to_vec(),
            }],
            statements: vec![],
        }))
    );
}

#[test]
fn parses_multiple_scalar_declarations_in_source_order() {
    let tokens = vec![
        Token::new(TokenType::LBrace, 1),
        Token::new(TokenType::Int, 2),
        Token::new(TokenType::Id(b"count".to_vec()), 2),
        Token::new(TokenType::SemiColon, 2),
        Token::new(TokenType::Char, 3),
        Token::new(TokenType::Id(b"letter".to_vec()), 3),
        Token::new(TokenType::SemiColon, 3),
        Token::new(TokenType::RBrace, 4),
        Token::new(TokenType::Eof, 4),
    ];

    let mut parser = Parser::new(tokens);

    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::Block(Block {
            declarations: vec![
                VariableDeclaration::Scalar {
                    data_type: DataType::Int,
                    name: b"count".to_vec(),
                },
                VariableDeclaration::Scalar {
                    data_type: DataType::Char,
                    name: b"letter".to_vec(),
                },
            ],
            statements: vec![],
        }))
    );
}

#[test]
fn parses_declarations_before_statements() {
    let tokens = vec![
        Token::new(TokenType::LBrace, 1),
        Token::new(TokenType::Int, 2),
        Token::new(TokenType::Id(b"x".to_vec()), 2),
        Token::new(TokenType::SemiColon, 2),
        Token::new(TokenType::Id(b"x".to_vec()), 3),
        Token::new(TokenType::Assign, 3),
        Token::new(TokenType::IntegerConst(10), 3),
        Token::new(TokenType::SemiColon, 3),
        Token::new(TokenType::RBrace, 4),
        Token::new(TokenType::Eof, 4),
    ];

    let mut parser = Parser::new(tokens);

    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::Block(Block {
            declarations: vec![VariableDeclaration::Scalar {
                data_type: DataType::Int,
                name: b"x".to_vec(),
            }],
            statements: vec![Statement::Assignment {
                target: AssignmentTarget::Identifier(b"x".to_vec()),
                value: Expression::Integer(10),
            }],
        }))
    );
}

#[test]
fn scalar_declaration_requires_identifier() {
    let tokens = vec![
        Token::new(TokenType::LBrace, 1),
        Token::new(TokenType::Int, 2),
        Token::new(TokenType::SemiColon, 2),
        Token::new(TokenType::RBrace, 3),
        Token::new(TokenType::Eof, 3),
    ];

    let mut parser = Parser::new(tokens);

    assert_eq!(
        parser.parse_statement(),
        Err(ParserError::UnexpectedToken {
            expected: "Id",
            found: TokenType::SemiColon,
            line: 2,
        })
    );
}

#[test]
fn scalar_declaration_requires_semicolon() {
    let tokens = vec![
        Token::new(TokenType::LBrace, 1),
        Token::new(TokenType::Int, 2),
        Token::new(TokenType::Id(b"x".to_vec()), 2),
        Token::new(TokenType::RBrace, 3),
        Token::new(TokenType::Eof, 3),
    ];

    let mut parser = Parser::new(tokens);

    assert_eq!(
        parser.parse_statement(),
        Err(ParserError::UnexpectedToken {
            expected: "';'",
            found: TokenType::RBrace,
            line: 3,
        })
    );
}

#[test]
fn parses_array_declaration() {
    let tokens = vec![
        Token::new(TokenType::LBrace, 1),
        Token::new(TokenType::Int, 2),
        Token::new(TokenType::Id(b"values".to_vec()), 2),
        Token::new(TokenType::LBracket, 2),
        Token::new(TokenType::IntegerConst(10), 2),
        Token::new(TokenType::RBracket, 2),
        Token::new(TokenType::SemiColon, 2),
        Token::new(TokenType::RBrace, 3),
        Token::new(TokenType::Eof, 3),
    ];

    let mut parser = Parser::new(tokens);

    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::Block(Block {
            declarations: vec![VariableDeclaration::Array {
                data_type: DataType::Int,
                name: b"values".to_vec(),
                length: 10,
            }],
            statements: vec![],
        }))
    );
}

#[test]
fn declaration_after_statement_is_rejected() {
    let tokens = vec![
        Token::new(TokenType::LBrace, 1),
        Token::new(TokenType::Id(b"x".to_vec()), 2),
        Token::new(TokenType::Assign, 2),
        Token::new(TokenType::IntegerConst(1), 2),
        Token::new(TokenType::SemiColon, 2),
        Token::new(TokenType::Int, 3),
        Token::new(TokenType::Id(b"y".to_vec()), 3),
        Token::new(TokenType::SemiColon, 3),
        Token::new(TokenType::RBrace, 4),
        Token::new(TokenType::Eof, 4),
    ];

    let mut parser = Parser::new(tokens);

    assert_eq!(
        parser.parse_statement(),
        Err(ParserError::UnexpectedToken {
            expected: "statement",
            found: TokenType::Int,
            line: 3,
        })
    );
}

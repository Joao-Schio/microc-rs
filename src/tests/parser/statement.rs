use crate::{
    ast::{
        expression::{BinaryOp, Expression},
        statement::{AssignmentTarget, Statement},
    },
    parser::{Parser, ParserError},
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
fn parses_array_element_assignment() {
    let tokens = vec![
        Token::new(TokenType::Id, 1, b"values".to_vec()),
        Token::new(TokenType::LBracket, 1, b"[".to_vec()),
        Token::new(TokenType::Id, 1, b"i".to_vec()),
        Token::new(TokenType::Plus, 1, b"+".to_vec()),
        Token::new(TokenType::IntegerConst(1), 1, b"1".to_vec()),
        Token::new(TokenType::RBracket, 1, b"]".to_vec()),
        Token::new(TokenType::Assign, 1, b"=".to_vec()),
        Token::new(TokenType::IntegerConst(42), 1, b"42".to_vec()),
        Token::new(TokenType::SemiColon, 1, b";".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    let mut parser = Parser::new(&tokens);

    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::Assignment {
            target: AssignmentTarget::ArrayElement {
                array: b"values".to_vec(),
                index: Box::new(Expression::Binary {
                    left: Box::new(Expression::Identifier(b"i".to_vec())),
                    op: BinaryOp::Add,
                    right: Box::new(Expression::Integer(1)),
                }),
            },
            value: Expression::Integer(42),
        })
    );
}

#[test]
fn assignment_requires_semicolon() {
    let tokens = vec![
        Token::new(TokenType::Id, 1, b"x".to_vec()),
        Token::new(TokenType::Assign, 1, b"=".to_vec()),
        Token::new(TokenType::IntegerConst(42), 1, b"42".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    let mut parser = Parser::new(&tokens);

    assert_eq!(
        parser.parse_statement(),
        Err(ParserError::UnexpectedToken {
            expected: "';'",
            found: TokenType::Eof,
            line: 1,
        })
    );
}

#[test]
fn array_assignment_requires_closing_bracket() {
    let tokens = vec![
        Token::new(TokenType::Id, 1, b"values".to_vec()),
        Token::new(TokenType::LBracket, 1, b"[".to_vec()),
        Token::new(TokenType::Id, 1, b"i".to_vec()),
        Token::new(TokenType::Assign, 1, b"=".to_vec()),
        Token::new(TokenType::IntegerConst(42), 1, b"42".to_vec()),
        Token::new(TokenType::SemiColon, 1, b";".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    let mut parser = Parser::new(&tokens);

    assert_eq!(
        parser.parse_statement(),
        Err(ParserError::UnexpectedToken {
            expected: "']'",
            found: TokenType::Assign,
            line: 1,
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

#[test]
fn parses_return_without_value() {
    let tokens = vec![
        Token::new(TokenType::Return, 1, b"return".to_vec()),
        Token::new(TokenType::SemiColon, 1, b";".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    let mut parser = Parser::new(&tokens);

    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::Return { value: None })
    );
}

#[test]
fn parses_return_with_value() {
    let tokens = vec![
        Token::new(TokenType::Return, 1, b"return".to_vec()),
        Token::new(TokenType::IntegerConst(42), 1, b"42".to_vec()),
        Token::new(TokenType::SemiColon, 1, b";".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    let mut parser = Parser::new(&tokens);

    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::Return {
            value: Some(Expression::Integer(42)),
        })
    );
}

#[test]
fn parses_expression_in_return() {
    let tokens = vec![
        Token::new(TokenType::Return, 1, b"return".to_vec()),
        Token::new(TokenType::Id, 1, b"x".to_vec()),
        Token::new(TokenType::Plus, 1, b"+".to_vec()),
        Token::new(TokenType::Id, 1, b"foo".to_vec()),
        Token::new(TokenType::Lparen, 1, b"(".to_vec()),
        Token::new(TokenType::IntegerConst(1), 1, b"1".to_vec()),
        Token::new(TokenType::Rparen, 1, b")".to_vec()),
        Token::new(TokenType::SemiColon, 1, b";".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    let mut parser = Parser::new(&tokens);

    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::Return {
            value: Some(Expression::Binary {
                left: Box::new(Expression::Identifier(b"x".to_vec())),
                op: BinaryOp::Add,
                right: Box::new(Expression::Call {
                    callee: b"foo".to_vec(),
                    arguments: vec![Expression::Integer(1)],
                }),
            }),
        })
    );
}

#[test]
fn return_requires_semicolon() {
    let tokens = vec![
        Token::new(TokenType::Return, 1, b"return".to_vec()),
        Token::new(TokenType::IntegerConst(42), 1, b"42".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    let mut parser = Parser::new(&tokens);

    assert_eq!(
        parser.parse_statement(),
        Err(ParserError::UnexpectedToken {
            expected: "';'",
            found: TokenType::Eof,
            line: 1,
        })
    );
}

#[test]
fn parses_assignment_followed_by_return() {
    let tokens = vec![
        Token::new(TokenType::Id, 1, b"x".to_vec()),
        Token::new(TokenType::Assign, 1, b"=".to_vec()),
        Token::new(TokenType::IntegerConst(1), 1, b"1".to_vec()),
        Token::new(TokenType::SemiColon, 1, b";".to_vec()),
        Token::new(TokenType::Return, 2, b"return".to_vec()),
        Token::new(TokenType::Id, 2, b"x".to_vec()),
        Token::new(TokenType::SemiColon, 2, b";".to_vec()),
        Token::new(TokenType::Eof, 2, vec![]),
    ];

    let mut parser = Parser::new(&tokens);

    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::Assignment {
            target: AssignmentTarget::Identifier(b"x".to_vec()),
            value: Expression::Integer(1),
        })
    );
    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::Return {
            value: Some(Expression::Identifier(b"x".to_vec())),
        })
    );
}

#[test]
fn rejects_invalid_statement_start() {
    let tokens = vec![
        Token::new(TokenType::Plus, 7, b"+".to_vec()),
        Token::new(TokenType::Eof, 7, vec![]),
    ];

    let mut parser = Parser::new(&tokens);

    assert_eq!(
        parser.parse_statement(),
        Err(ParserError::UnexpectedToken {
            expected: "statement",
            found: TokenType::Plus,
            line: 7,
        })
    );
}

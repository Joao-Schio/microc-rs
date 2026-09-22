use crate::{
    ast::{
        expression::Expression,
        statement::{AssignmentTarget, Statement},
    },
    parser::{Parser, ParserError},
    token::{Token, TokenType},
};

#[test]
fn parses_if_without_else() {
    let tokens = vec![
        Token::new(TokenType::If, 1, b"if".to_vec()),
        Token::new(TokenType::Lparen, 1, b"(".to_vec()),
        Token::new(TokenType::Id, 1, b"x".to_vec()),
        Token::new(TokenType::Rparen, 1, b")".to_vec()),
        Token::new(TokenType::Return, 1, b"return".to_vec()),
        Token::new(TokenType::IntegerConst(1), 1, b"1".to_vec()),
        Token::new(TokenType::SemiColon, 1, b";".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    let mut parser = Parser::new(tokens);

    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::If {
            condition: Expression::Identifier(b"x".to_vec()),
            then_branch: Box::new(Statement::Return {
                value: Some(Expression::Integer(1)),
            }),
            else_branch: None,
        })
    );
}

#[test]
fn parses_if_with_else() {
    let tokens = vec![
        Token::new(TokenType::If, 1, b"if".to_vec()),
        Token::new(TokenType::Lparen, 1, b"(".to_vec()),
        Token::new(TokenType::Id, 1, b"x".to_vec()),
        Token::new(TokenType::Rparen, 1, b")".to_vec()),
        Token::new(TokenType::Return, 1, b"return".to_vec()),
        Token::new(TokenType::IntegerConst(1), 1, b"1".to_vec()),
        Token::new(TokenType::SemiColon, 1, b";".to_vec()),
        Token::new(TokenType::Else, 1, b"else".to_vec()),
        Token::new(TokenType::Return, 1, b"return".to_vec()),
        Token::new(TokenType::IntegerConst(2), 1, b"2".to_vec()),
        Token::new(TokenType::SemiColon, 1, b";".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    let mut parser = Parser::new(tokens);

    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::If {
            condition: Expression::Identifier(b"x".to_vec()),
            then_branch: Box::new(Statement::Return {
                value: Some(Expression::Integer(1)),
            }),
            else_branch: Some(Box::new(Statement::Return {
                value: Some(Expression::Integer(2)),
            })),
        })
    );
}

#[test]
fn if_requires_opening_parenthesis() {
    let tokens = vec![
        Token::new(TokenType::If, 1, b"if".to_vec()),
        Token::new(TokenType::Id, 1, b"x".to_vec()),
        Token::new(TokenType::Rparen, 1, b")".to_vec()),
        Token::new(TokenType::Return, 1, b"return".to_vec()),
        Token::new(TokenType::IntegerConst(1), 1, b"1".to_vec()),
        Token::new(TokenType::SemiColon, 1, b";".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    let mut parser = Parser::new(tokens);

    assert_eq!(
        parser.parse_statement(),
        Err(ParserError::UnexpectedToken {
            expected: "'('",
            found: TokenType::Id,
            line: 1,
        })
    );
}

#[test]
fn if_requires_closing_parenthesis() {
    let tokens = vec![
        Token::new(TokenType::If, 1, b"if".to_vec()),
        Token::new(TokenType::Lparen, 1, b"(".to_vec()),
        Token::new(TokenType::Id, 1, b"x".to_vec()),
        Token::new(TokenType::Return, 1, b"return".to_vec()),
        Token::new(TokenType::IntegerConst(1), 1, b"1".to_vec()),
        Token::new(TokenType::SemiColon, 1, b";".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    let mut parser = Parser::new(tokens);

    assert_eq!(
        parser.parse_statement(),
        Err(ParserError::UnexpectedToken {
            expected: "')'",
            found: TokenType::Return,
            line: 1,
        })
    );
}

#[test]
fn else_binds_to_nearest_if() {
    let tokens = vec![
        Token::new(TokenType::If, 1, b"if".to_vec()),
        Token::new(TokenType::Lparen, 1, b"(".to_vec()),
        Token::new(TokenType::Id, 1, b"a".to_vec()),
        Token::new(TokenType::Rparen, 1, b")".to_vec()),
        Token::new(TokenType::If, 2, b"if".to_vec()),
        Token::new(TokenType::Lparen, 2, b"(".to_vec()),
        Token::new(TokenType::Id, 2, b"b".to_vec()),
        Token::new(TokenType::Rparen, 2, b")".to_vec()),
        Token::new(TokenType::Return, 3, b"return".to_vec()),
        Token::new(TokenType::IntegerConst(1), 3, b"1".to_vec()),
        Token::new(TokenType::SemiColon, 3, b";".to_vec()),
        Token::new(TokenType::Else, 4, b"else".to_vec()),
        Token::new(TokenType::Return, 4, b"return".to_vec()),
        Token::new(TokenType::IntegerConst(2), 4, b"2".to_vec()),
        Token::new(TokenType::SemiColon, 4, b";".to_vec()),
        Token::new(TokenType::Eof, 4, vec![]),
    ];

    let mut parser = Parser::new(tokens);

    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::If {
            condition: Expression::Identifier(b"a".to_vec()),
            then_branch: Box::new(Statement::If {
                condition: Expression::Identifier(b"b".to_vec()),
                then_branch: Box::new(Statement::Return {
                    value: Some(Expression::Integer(1)),
                }),
                else_branch: Some(Box::new(Statement::Return {
                    value: Some(Expression::Integer(2)),
                })),
            }),
            else_branch: None,
        })
    );
}

#[test]
fn if_accepts_empty_statement_as_body() {
    let tokens = vec![
        Token::new(TokenType::If, 1, b"if".to_vec()),
        Token::new(TokenType::Lparen, 1, b"(".to_vec()),
        Token::new(TokenType::Id, 1, b"x".to_vec()),
        Token::new(TokenType::Rparen, 1, b")".to_vec()),
        Token::new(TokenType::SemiColon, 1, b";".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    let mut parser = Parser::new(tokens);

    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::If {
            condition: Expression::Identifier(b"x".to_vec()),
            then_branch: Box::new(Statement::Empty),
            else_branch: None,
        })
    );
}

#[test]
fn else_requires_statement() {
    let tokens = vec![
        Token::new(TokenType::If, 1, b"if".to_vec()),
        Token::new(TokenType::Lparen, 1, b"(".to_vec()),
        Token::new(TokenType::Id, 1, b"x".to_vec()),
        Token::new(TokenType::Rparen, 1, b")".to_vec()),
        Token::new(TokenType::Return, 1, b"return".to_vec()),
        Token::new(TokenType::IntegerConst(1), 1, b"1".to_vec()),
        Token::new(TokenType::SemiColon, 1, b";".to_vec()),
        Token::new(TokenType::Else, 1, b"else".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    let mut parser = Parser::new(tokens);

    assert_eq!(
        parser.parse_statement(),
        Err(ParserError::UnexpectedToken {
            expected: "statement",
            found: TokenType::Eof,
            line: 1,
        })
    );
}

#[test]
fn if_consumes_only_its_statement() {
    let tokens = vec![
        Token::new(TokenType::If, 1, b"if".to_vec()),
        Token::new(TokenType::Lparen, 1, b"(".to_vec()),
        Token::new(TokenType::Id, 1, b"x".to_vec()),
        Token::new(TokenType::Rparen, 1, b")".to_vec()),
        Token::new(TokenType::Return, 1, b"return".to_vec()),
        Token::new(TokenType::IntegerConst(1), 1, b"1".to_vec()),
        Token::new(TokenType::SemiColon, 1, b";".to_vec()),
        Token::new(TokenType::Id, 2, b"y".to_vec()),
        Token::new(TokenType::Assign, 2, b"=".to_vec()),
        Token::new(TokenType::IntegerConst(2), 2, b"2".to_vec()),
        Token::new(TokenType::SemiColon, 2, b";".to_vec()),
        Token::new(TokenType::Eof, 2, vec![]),
    ];

    let mut parser = Parser::new(tokens);

    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::If {
            condition: Expression::Identifier(b"x".to_vec()),
            then_branch: Box::new(Statement::Return {
                value: Some(Expression::Integer(1)),
            }),
            else_branch: None,
        })
    );

    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::Assignment {
            target: AssignmentTarget::Identifier(b"y".to_vec()),
            value: Expression::Integer(2),
        })
    );
}

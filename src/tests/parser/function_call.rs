use crate::{
    ast::expression::{BinaryOp, Expression},
    parser::{ExprParser, Parser, ParserError},
    token::{Token, TokenType},
};

fn parse(tokens: &[Token]) -> Result<Expression, ParserError> {
    let mut parser = Parser::with_expr_parser(tokens, ExprParser);
    parser.parse_expression()
}

#[test]
fn parses_function_call_with_one_argument() {
    let tokens = vec![
        Token::new(TokenType::Id, 1, b"foo".to_vec()),
        Token::new(TokenType::Lparen, 1, b"(".to_vec()),
        Token::new(TokenType::IntegerConst(1), 1, b"1".to_vec()),
        Token::new(TokenType::Rparen, 1, b")".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    assert_eq!(
        parse(&tokens),
        Ok(Expression::Call {
            callee: b"foo".to_vec(),
            arguments: vec![Expression::Integer(1)],
        })
    );
}

#[test]
fn parses_function_call_with_multiple_arguments() {
    let tokens = vec![
        Token::new(TokenType::Id, 1, b"foo".to_vec()),
        Token::new(TokenType::Lparen, 1, b"(".to_vec()),
        Token::new(TokenType::IntegerConst(1), 1, b"1".to_vec()),
        Token::new(TokenType::Comma, 1, b",".to_vec()),
        Token::new(TokenType::IntegerConst(2), 1, b"2".to_vec()),
        Token::new(TokenType::Comma, 1, b",".to_vec()),
        Token::new(TokenType::IntegerConst(3), 1, b"3".to_vec()),
        Token::new(TokenType::Rparen, 1, b")".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    assert_eq!(
        parse(&tokens),
        Ok(Expression::Call {
            callee: b"foo".to_vec(),
            arguments: vec![
                Expression::Integer(1),
                Expression::Integer(2),
                Expression::Integer(3),
            ],
        })
    );
}

#[test]
fn parses_expression_as_function_argument() {
    let tokens = vec![
        Token::new(TokenType::Id, 1, b"foo".to_vec()),
        Token::new(TokenType::Lparen, 1, b"(".to_vec()),
        Token::new(TokenType::IntegerConst(1), 1, b"1".to_vec()),
        Token::new(TokenType::Plus, 1, b"+".to_vec()),
        Token::new(TokenType::IntegerConst(2), 1, b"2".to_vec()),
        Token::new(TokenType::Rparen, 1, b")".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    assert_eq!(
        parse(&tokens),
        Ok(Expression::Call {
            callee: b"foo".to_vec(),
            arguments: vec![Expression::Binary {
                left: Box::new(Expression::Integer(1)),
                op: BinaryOp::Add,
                right: Box::new(Expression::Integer(2)),
            }],
        })
    );
}

#[test]
fn parses_nested_function_call() {
    let tokens = vec![
        Token::new(TokenType::Id, 1, b"outer".to_vec()),
        Token::new(TokenType::Lparen, 1, b"(".to_vec()),
        Token::new(TokenType::Id, 1, b"inner".to_vec()),
        Token::new(TokenType::Lparen, 1, b"(".to_vec()),
        Token::new(TokenType::IntegerConst(1), 1, b"1".to_vec()),
        Token::new(TokenType::Rparen, 1, b")".to_vec()),
        Token::new(TokenType::Rparen, 1, b")".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    assert_eq!(
        parse(&tokens),
        Ok(Expression::Call {
            callee: b"outer".to_vec(),
            arguments: vec![Expression::Call {
                callee: b"inner".to_vec(),
                arguments: vec![Expression::Integer(1)],
            }],
        })
    );
}

#[test]
fn function_call_requires_closing_parenthesis() {
    let tokens = vec![
        Token::new(TokenType::Id, 1, b"foo".to_vec()),
        Token::new(TokenType::Lparen, 1, b"(".to_vec()),
        Token::new(TokenType::IntegerConst(1), 1, b"1".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    assert_eq!(
        parse(&tokens),
        Err(ParserError::UnexpectedToken {
            expected: "')'",
            found: TokenType::Eof,
            line: 1,
        })
    );
}

#[test]
fn function_call_requires_expression_after_comma() {
    let tokens = vec![
        Token::new(TokenType::Id, 1, b"foo".to_vec()),
        Token::new(TokenType::Lparen, 1, b"(".to_vec()),
        Token::new(TokenType::IntegerConst(1), 1, b"1".to_vec()),
        Token::new(TokenType::Comma, 1, b",".to_vec()),
        Token::new(TokenType::Rparen, 1, b")".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    assert_eq!(
        parse(&tokens),
        Err(ParserError::UnexpectedToken {
            expected: "expression",
            found: TokenType::Rparen,
            line: 1,
        })
    );
}

#[test]
fn zero_argument_call_composes_in_arithmetic_expression() {
    let tokens = vec![
        Token::new(TokenType::Id, 1, b"foo".to_vec()),
        Token::new(TokenType::Lparen, 1, b"(".to_vec()),
        Token::new(TokenType::Rparen, 1, b")".to_vec()),
        Token::new(TokenType::Plus, 1, b"+".to_vec()),
        Token::new(TokenType::IntegerConst(1), 1, b"1".to_vec()),
        Token::new(TokenType::Eof, 1, vec![]),
    ];

    assert_eq!(
        parse(&tokens),
        Ok(Expression::Binary {
            left: Box::new(Expression::Call {
                callee: b"foo".to_vec(),
                arguments: vec![],
            }),
            op: BinaryOp::Add,
            right: Box::new(Expression::Integer(1)),
        })
    );
}

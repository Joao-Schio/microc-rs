use crate::{
    ast::expression::{BinaryOp, Expression, UnaryOp},
    parser::{ExprParser, Parser, ParserContext, ParserError, TExprParser},
    token::{Token, TokenType},
};

macro_rules! expression_parser_contract {
    ($module:ident, $parser:ty) => {
        mod $module {
            use super::*;

            fn parse(tokens: &[Token]) -> Result<Expression, ParserError> {
                let mut parser = Parser::with_expr_parser(tokens, <$parser>::default());
                parser.parse_expression()
            }

            #[test]
            fn parses_integer_expression() {
                let tokens = vec![
                    Token::new(TokenType::IntegerConst(42), 1, b"42".to_vec()),
                    Token::new(TokenType::Eof, 1, vec![]),
                ];

                assert_eq!(parse(&tokens), Ok(Expression::Integer(42)));
            }

            #[test]
            fn non_expression_token_returns_parser_error() {
                let tokens = vec![Token::new(TokenType::Plus, 1, b"+".to_vec())];

                assert_eq!(
                    parse(&tokens),
                    Err(ParserError::UnexpectedToken {
                        expected: "expression",
                        found: TokenType::Plus,
                        line: 1
                    })
                );
            }

            #[test]
            fn parses_identifier_expression() {
                let tokens = vec![
                    Token::new(TokenType::Id, 1, b"value".to_vec()),
                    Token::new(TokenType::Eof, 1, vec![]),
                ];

                assert_eq!(
                    parse(&tokens),
                    Ok(Expression::Identifier(b"value".to_vec()))
                );
            }

            #[test]
            fn parses_char_expression() {
                let tokens = vec![
                    Token::new(TokenType::CharConst, 1, b"a".to_vec()),
                    Token::new(TokenType::Eof, 1, vec![]),
                ];

                assert_eq!(parse(&tokens), Ok(Expression::Char(b'a')));
            }

            #[test]
            fn parses_parenthesized_expression() {
                let tokens = vec![
                    Token::new(TokenType::Lparen, 1, b"(".to_vec()),
                    Token::new(TokenType::IntegerConst(42), 1, b"42".to_vec()),
                    Token::new(TokenType::Rparen, 1, b")".to_vec()),
                    Token::new(TokenType::Eof, 1, vec![]),
                ];

                assert_eq!(parse(&tokens), Ok(Expression::Integer(42)));
            }

            #[test]
            fn parenthesized_expression_requires_closing_parenthesis() {
                let tokens = vec![
                    Token::new(TokenType::Lparen, 1, b"(".to_vec()),
                    Token::new(TokenType::IntegerConst(42), 1, b"42".to_vec()),
                    Token::new(TokenType::Eof, 1, vec![]),
                ];

                assert_eq!(
                    parse(&tokens),
                    Err(ParserError::UnexpectedToken {
                        expected: "')'",
                        found: TokenType::Eof,
                        line: 1
                    })
                );
            }

            #[test]
            fn parses_logical_not_expression() {
                let tokens = vec![
                    Token::new(TokenType::Not, 1, b"!".to_vec()),
                    Token::new(TokenType::IntegerConst(42), 1, b"42".to_vec()),
                    Token::new(TokenType::Eof, 1, vec![]),
                ];

                assert_eq!(
                    parse(&tokens),
                    Ok(Expression::Unary {
                        op: UnaryOp::Not,
                        expression: Box::new(Expression::Integer(42)),
                    })
                );
            }

            #[test]
            fn parses_negative_number() {
                let tokens = vec![
                    Token::new(TokenType::Minus, 1, b"-".to_vec()),
                    Token::new(TokenType::IntegerConst(42), 1, b"42".to_vec()),
                    Token::new(TokenType::Eof, 1, vec![]),
                ];

                assert_eq!(
                    parse(&tokens),
                    Ok(Expression::Unary {
                        op: UnaryOp::Negate,
                        expression: Box::new(Expression::Integer(42)),
                    })
                );
            }

            #[test]
            fn parses_chained_unary_expression() {
                let tokens = vec![
                    Token::new(TokenType::Minus, 1, b"-".to_vec()),
                    Token::new(TokenType::Not, 1, b"!".to_vec()),
                    Token::new(TokenType::Id, 1, b"value".to_vec()),
                    Token::new(TokenType::Eof, 1, vec![]),
                ];

                assert_eq!(
                    parse(&tokens),
                    Ok(Expression::Unary {
                        op: UnaryOp::Negate,
                        expression: Box::new(Expression::Unary {
                            op: UnaryOp::Not,
                            expression: Box::new(Expression::Identifier(b"value".to_vec())),
                        }),
                    })
                );
            }

            #[test]
            fn parses_multiplication_expression() {
                let tokens = vec![
                    Token::new(TokenType::IntegerConst(2), 1, b"2".to_vec()),
                    Token::new(TokenType::Mul, 1, b"*".to_vec()),
                    Token::new(TokenType::IntegerConst(3), 1, b"3".to_vec()),
                    Token::new(TokenType::Eof, 1, vec![]),
                ];

                assert_eq!(
                    parse(&tokens),
                    Ok(Expression::Binary {
                        left: Box::new(Expression::Integer(2)),
                        op: BinaryOp::Multiply,
                        right: Box::new(Expression::Integer(3)),
                    })
                );
            }

            #[test]
            fn multiplicative_operators_are_left_associative() {
                let tokens = vec![
                    Token::new(TokenType::IntegerConst(8), 1, b"8".to_vec()),
                    Token::new(TokenType::Div, 1, b"/".to_vec()),
                    Token::new(TokenType::IntegerConst(4), 1, b"4".to_vec()),
                    Token::new(TokenType::Mul, 1, b"*".to_vec()),
                    Token::new(TokenType::IntegerConst(2), 1, b"2".to_vec()),
                    Token::new(TokenType::Eof, 1, vec![]),
                ];

                assert_eq!(
                    parse(&tokens),
                    Ok(Expression::Binary {
                        left: Box::new(Expression::Binary {
                            left: Box::new(Expression::Integer(8)),
                            op: BinaryOp::Divide,
                            right: Box::new(Expression::Integer(4)),
                        }),
                        op: BinaryOp::Multiply,
                        right: Box::new(Expression::Integer(2)),
                    })
                );
            }

            #[test]
            fn multiplication_has_higher_precedence_than_addition() {
                let tokens = vec![
                    Token::new(TokenType::IntegerConst(1), 1, b"1".to_vec()),
                    Token::new(TokenType::Plus, 1, b"+".to_vec()),
                    Token::new(TokenType::IntegerConst(2), 1, b"2".to_vec()),
                    Token::new(TokenType::Mul, 1, b"*".to_vec()),
                    Token::new(TokenType::IntegerConst(3), 1, b"3".to_vec()),
                    Token::new(TokenType::Eof, 1, vec![]),
                ];

                assert_eq!(
                    parse(&tokens),
                    Ok(Expression::Binary {
                        left: Box::new(Expression::Integer(1)),
                        op: BinaryOp::Add,
                        right: Box::new(Expression::Binary {
                            left: Box::new(Expression::Integer(2)),
                            op: BinaryOp::Multiply,
                            right: Box::new(Expression::Integer(3)),
                        }),
                    })
                );
            }

            #[test]
            fn parses_relational_expression() {
                let tokens = vec![
                    Token::new(TokenType::IntegerConst(1), 1, b"1".to_vec()),
                    Token::new(TokenType::Lt, 1, b"<".to_vec()),
                    Token::new(TokenType::IntegerConst(2), 1, b"2".to_vec()),
                    Token::new(TokenType::Eof, 1, vec![]),
                ];

                assert_eq!(
                    parse(&tokens),
                    Ok(Expression::Binary {
                        left: Box::new(Expression::Integer(1)),
                        op: BinaryOp::Less,
                        right: Box::new(Expression::Integer(2)),
                    })
                );
            }

            #[test]
            fn relational_operators_have_higher_precedence_than_logical_operators() {
                let tokens = vec![
                    Token::new(TokenType::IntegerConst(1), 1, b"1".to_vec()),
                    Token::new(TokenType::Lt, 1, b"<".to_vec()),
                    Token::new(TokenType::IntegerConst(2), 1, b"2".to_vec()),
                    Token::new(TokenType::And, 1, b"&&".to_vec()),
                    Token::new(TokenType::IntegerConst(3), 1, b"3".to_vec()),
                    Token::new(TokenType::Lt, 1, b"<".to_vec()),
                    Token::new(TokenType::IntegerConst(4), 1, b"4".to_vec()),
                    Token::new(TokenType::Eof, 1, vec![]),
                ];

                assert_eq!(
                    parse(&tokens),
                    Ok(Expression::Binary {
                        left: Box::new(Expression::Binary {
                            left: Box::new(Expression::Integer(1)),
                            op: BinaryOp::Less,
                            right: Box::new(Expression::Integer(2)),
                        }),
                        op: BinaryOp::And,
                        right: Box::new(Expression::Binary {
                            left: Box::new(Expression::Integer(3)),
                            op: BinaryOp::Less,
                            right: Box::new(Expression::Integer(4)),
                        }),
                    })
                );
            }

            #[test]
            fn parses_array_access_expression() {
                let tokens = vec![
                    Token::new(TokenType::Id, 1, b"values".to_vec()),
                    Token::new(TokenType::LBracket, 1, b"[".to_vec()),
                    Token::new(TokenType::IntegerConst(2), 1, b"2".to_vec()),
                    Token::new(TokenType::Plus, 1, b"+".to_vec()),
                    Token::new(TokenType::IntegerConst(3), 1, b"3".to_vec()),
                    Token::new(TokenType::RBracket, 1, b"]".to_vec()),
                    Token::new(TokenType::Eof, 1, vec![]),
                ];

                assert_eq!(
                    parse(&tokens),
                    Ok(Expression::ArrayAccess {
                        array: b"values".to_vec(),
                        index: Box::new(Expression::Binary {
                            left: Box::new(Expression::Integer(2)),
                            op: BinaryOp::Add,
                            right: Box::new(Expression::Integer(3)),
                        }),
                    })
                );
            }

            #[test]
            fn array_access_requires_closing_bracket() {
                let tokens = vec![
                    Token::new(TokenType::Id, 1, b"values".to_vec()),
                    Token::new(TokenType::LBracket, 1, b"[".to_vec()),
                    Token::new(TokenType::IntegerConst(2), 1, b"2".to_vec()),
                    Token::new(TokenType::Eof, 1, vec![]),
                ];

                assert_eq!(
                    parse(&tokens),
                    Err(ParserError::UnexpectedToken {
                        expected: "']'",
                        found: TokenType::Eof,
                        line: 1
                    })
                );
            }

            #[test]
            fn parses_function_call_without_arguments() {
                let tokens = vec![
                    Token::new(TokenType::Id, 1, b"foo".to_vec()),
                    Token::new(TokenType::Lparen, 1, b"(".to_vec()),
                    Token::new(TokenType::Rparen, 1, b")".to_vec()),
                    Token::new(TokenType::Eof, 1, vec![]),
                ];

                assert_eq!(
                    parse(&tokens),
                    Ok(Expression::Call {
                        callee: b"foo".to_vec(),
                        arguments: vec![],
                    })
                );
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
        }
    };
}

expression_parser_contract!(expr_parser, ExprParser);

struct StubExprParser;

impl TExprParser for StubExprParser {
    fn parse_expression(
        &mut self,
        _context: &mut ParserContext<'_>,
    ) -> Result<Expression, ParserError> {
        Ok(Expression::Integer(99))
    }
}

#[test]
fn parser_uses_injected_expression_parser() {
    let tokens = vec![Token::new(TokenType::Eof, 1, vec![])];
    let mut parser = Parser::with_expr_parser(&tokens, StubExprParser);

    assert_eq!(parser.parse_expression(), Ok(Expression::Integer(99)));
}

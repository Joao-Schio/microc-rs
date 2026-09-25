use crate::{
    ast::{
        expression::{BinaryOp, Expression},
        statement::{Assignment, LValue, PrintContent, Statement, StatementKind},
    },
    parser::{ExprParser, Parser, ParserContext, ParserError, TExprParser, TStatementParser},
    token::{Token, TokenType},
};

parser_contract!(
    default_statement_parser,
    crate::parser::ExprParser,
    crate::parser::StatementParser,
    {
        #[test]
        fn parses_identifier_assignment() {
            let tokens = vec![
                Token::new(TokenType::Id(b"x".to_vec()), 1),
                Token::new(TokenType::Assign, 1),
                Token::new(TokenType::IntegerConst(42), 1),
                Token::new(TokenType::SemiColon, 1),
                Token::new(TokenType::Eof, 1),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Ok(statement!(
                    1,
                    StatementKind::Assignment(Assignment {
                        target: LValue::Identifier(identifier!(b"x")),
                        value: Expression::Integer(42),
                    })
                ))
            );
        }

        #[test]
        fn parses_array_element_assignment() {
            let tokens = vec![
                Token::new(TokenType::Id(b"values".to_vec()), 1),
                Token::new(TokenType::LBracket, 1),
                Token::new(TokenType::Id(b"i".to_vec()), 1),
                Token::new(TokenType::Plus, 1),
                Token::new(TokenType::IntegerConst(1), 1),
                Token::new(TokenType::RBracket, 1),
                Token::new(TokenType::Assign, 1),
                Token::new(TokenType::IntegerConst(42), 1),
                Token::new(TokenType::SemiColon, 1),
                Token::new(TokenType::Eof, 1),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Ok(statement!(
                    1,
                    StatementKind::Assignment(Assignment {
                        target: LValue::ArrayElement {
                            array: identifier!(b"values"),
                            index: Box::new(Expression::Binary {
                                left: Box::new(Expression::Identifier(identifier!(b"i"))),
                                op: BinaryOp::Add,
                                right: Box::new(Expression::Integer(1)),
                            }),
                        },
                        value: Expression::Integer(42),
                    })
                ))
            );
        }

        #[test]
        fn assignment_requires_semicolon() {
            let tokens = vec![
                Token::new(TokenType::Id(b"x".to_vec()), 1),
                Token::new(TokenType::Assign, 1),
                Token::new(TokenType::IntegerConst(42), 1),
                Token::new(TokenType::Eof, 1),
            ];

            let mut parser = make_parser(tokens);

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
                Token::new(TokenType::Id(b"values".to_vec()), 1),
                Token::new(TokenType::LBracket, 1),
                Token::new(TokenType::Id(b"i".to_vec()), 1),
                Token::new(TokenType::Assign, 1),
                Token::new(TokenType::IntegerConst(42), 1),
                Token::new(TokenType::SemiColon, 1),
                Token::new(TokenType::Eof, 1),
            ];

            let mut parser = make_parser(tokens);

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
                Token::new(TokenType::Id(b"x".to_vec()), 1),
                Token::new(TokenType::Assign, 1),
                Token::new(TokenType::IntegerConst(1), 1),
                Token::new(TokenType::SemiColon, 1),
                Token::new(TokenType::Id(b"y".to_vec()), 2),
                Token::new(TokenType::Assign, 2),
                Token::new(TokenType::IntegerConst(2), 2),
                Token::new(TokenType::SemiColon, 2),
                Token::new(TokenType::Eof, 2),
            ];

            let mut parser = make_parser(tokens);

            assert!(parser.parse_statement().is_ok());
            assert!(parser.parse_statement().is_ok());
        }

        #[test]
        fn parses_return_without_value() {
            let tokens = vec![
                Token::new(TokenType::Return, 1),
                Token::new(TokenType::SemiColon, 1),
                Token::new(TokenType::Eof, 1),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Ok(statement!(1, StatementKind::Return { value: None }))
            );
        }

        #[test]
        fn parses_return_with_value() {
            let tokens = vec![
                Token::new(TokenType::Return, 1),
                Token::new(TokenType::IntegerConst(42), 1),
                Token::new(TokenType::SemiColon, 1),
                Token::new(TokenType::Eof, 1),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Ok(statement!(
                    1,
                    StatementKind::Return {
                        value: Some(Expression::Integer(42)),
                    }
                ))
            );
        }

        #[test]
        fn parses_expression_in_return() {
            let tokens = vec![
                Token::new(TokenType::Return, 1),
                Token::new(TokenType::Id(b"x".to_vec()), 1),
                Token::new(TokenType::Plus, 1),
                Token::new(TokenType::Id(b"foo".to_vec()), 1),
                Token::new(TokenType::Lparen, 1),
                Token::new(TokenType::IntegerConst(1), 1),
                Token::new(TokenType::Rparen, 1),
                Token::new(TokenType::SemiColon, 1),
                Token::new(TokenType::Eof, 1),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Ok(statement!(
                    1,
                    StatementKind::Return {
                        value: Some(Expression::Binary {
                            left: Box::new(Expression::Identifier(identifier!(b"x"))),
                            op: BinaryOp::Add,
                            right: Box::new(Expression::Call {
                                callee: identifier!(b"foo"),
                                arguments: vec![Expression::Integer(1)],
                            }),
                        }),
                    }
                ))
            );
        }

        #[test]
        fn return_requires_semicolon() {
            let tokens = vec![
                Token::new(TokenType::Return, 1),
                Token::new(TokenType::IntegerConst(42), 1),
                Token::new(TokenType::Eof, 1),
            ];

            let mut parser = make_parser(tokens);

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
                Token::new(TokenType::Id(b"x".to_vec()), 1),
                Token::new(TokenType::Assign, 1),
                Token::new(TokenType::IntegerConst(1), 1),
                Token::new(TokenType::SemiColon, 1),
                Token::new(TokenType::Return, 2),
                Token::new(TokenType::Id(b"x".to_vec()), 2),
                Token::new(TokenType::SemiColon, 2),
                Token::new(TokenType::Eof, 2),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Ok(statement!(
                    1,
                    StatementKind::Assignment(Assignment {
                        target: LValue::Identifier(identifier!(b"x")),
                        value: Expression::Integer(1),
                    })
                ))
            );
            assert_eq!(
                parser.parse_statement(),
                Ok(statement!(
                    2,
                    StatementKind::Return {
                        value: Some(Expression::Identifier(identifier!(b"x", 2))),
                    }
                ))
            );
        }

        #[test]
        fn rejects_invalid_statement_start() {
            let tokens = vec![
                Token::new(TokenType::Plus, 7),
                Token::new(TokenType::Eof, 7),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Err(ParserError::UnexpectedToken {
                    expected: "statement",
                    found: TokenType::Plus,
                    line: 7,
                })
            );
        }

        #[test]
        fn parses_print_string() {
            let tokens = vec![
                Token::new(TokenType::Print, 1),
                Token::new(TokenType::Lparen, 1),
                Token::new(TokenType::StringConst(b"hello".to_vec()), 1),
                Token::new(TokenType::Rparen, 1),
                Token::new(TokenType::SemiColon, 1),
                Token::new(TokenType::Eof, 1),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Ok(statement!(
                    1,
                    StatementKind::Print {
                        content: PrintContent::StringConst(b"hello".to_vec()),
                    }
                ))
            );
        }

        #[test]
        fn parses_print_expression() {
            let tokens = vec![
                Token::new(TokenType::Print, 1),
                Token::new(TokenType::Lparen, 1),
                Token::new(TokenType::Id(b"a".to_vec()), 1),
                Token::new(TokenType::Plus, 1),
                Token::new(TokenType::Id(b"b".to_vec()), 1),
                Token::new(TokenType::Rparen, 1),
                Token::new(TokenType::SemiColon, 1),
                Token::new(TokenType::Eof, 1),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Ok(statement!(
                    1,
                    StatementKind::Print {
                        content: PrintContent::Expression(Expression::Binary {
                            left: Box::new(Expression::Identifier(identifier!(b"a"))),
                            op: BinaryOp::Add,
                            right: Box::new(Expression::Identifier(identifier!(b"b"))),
                        }),
                    }
                ))
            );
        }

        #[test]
        fn print_requires_closing_parenthesis() {
            let tokens = vec![
                Token::new(TokenType::Print, 1),
                Token::new(TokenType::Lparen, 1),
                Token::new(TokenType::IntegerConst(42), 1),
                Token::new(TokenType::SemiColon, 1),
                Token::new(TokenType::Eof, 1),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Err(ParserError::UnexpectedToken {
                    expected: "')'",
                    found: TokenType::SemiColon,
                    line: 1,
                })
            );
        }

        #[test]
        fn print_requires_semicolon() {
            let tokens = vec![
                Token::new(TokenType::Print, 1),
                Token::new(TokenType::Lparen, 1),
                Token::new(TokenType::IntegerConst(42), 1),
                Token::new(TokenType::Rparen, 1),
                Token::new(TokenType::Eof, 1),
            ];

            let mut parser = make_parser(tokens);

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
        fn rejects_empty_print() {
            let tokens = vec![
                Token::new(TokenType::Print, 1),
                Token::new(TokenType::Lparen, 1),
                Token::new(TokenType::Rparen, 1),
                Token::new(TokenType::SemiColon, 1),
                Token::new(TokenType::Eof, 1),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Err(ParserError::UnexpectedToken {
                    expected: "expression",
                    found: TokenType::Rparen,
                    line: 1,
                })
            );
        }

        #[test]
        fn parses_empty_statement() {
            let tokens = vec![
                Token::new(TokenType::SemiColon, 1),
                Token::new(TokenType::Eof, 1),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Ok(statement!(1, StatementKind::Empty))
            );
        }

        #[test]
        fn empty_statement_consumes_only_its_semicolon() {
            let tokens = vec![
                Token::new(TokenType::SemiColon, 1),
                Token::new(TokenType::Id(b"x".to_vec()), 2),
                Token::new(TokenType::Assign, 2),
                Token::new(TokenType::IntegerConst(42), 2),
                Token::new(TokenType::SemiColon, 2),
                Token::new(TokenType::Eof, 2),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Ok(statement!(1, StatementKind::Empty))
            );

            assert_eq!(
                parser.parse_statement(),
                Ok(statement!(
                    2,
                    StatementKind::Assignment(Assignment {
                        target: LValue::Identifier(identifier!(b"x", 2)),
                        value: Expression::Integer(42),
                    })
                ))
            );
        }
    }
);

struct StubStatementParser;

impl<E: TExprParser> TStatementParser<E> for StubStatementParser {
    fn parse_statement(
        &mut self,
        context: &mut ParserContext,
        _expr_parser: &mut E,
    ) -> Result<Statement, ParserError> {
        Ok(Statement::new(context.current().line(), StatementKind::Empty))
    }
}

#[test]
fn parser_uses_injected_statement_parser() {
    let tokens = vec![Token::new(TokenType::Eof, 1)];
    let mut parser = Parser::with_parsers(tokens, ExprParser, StubStatementParser);

    assert_eq!(
        parser.parse_statement(),
        Ok(statement!(1, StatementKind::Empty))
    );
}

use crate::{
    ast::{
        expression::Expression,
        statement::{Assignment, LValue, StatementKind},
    },
    parser::ParserError,
    token::{Token, TokenType},
};

parser_contract!(
    default_statement_parser,
    crate::parser::ExprParser,
    crate::parser::StatementParser,
    {
        #[test]
        fn parses_if_without_else() {
            let tokens = vec![
                Token::new(TokenType::If, 1),
                Token::new(TokenType::Lparen, 1),
                Token::new(TokenType::Id(b"x".to_vec()), 1),
                Token::new(TokenType::Rparen, 1),
                Token::new(TokenType::Return, 1),
                Token::new(TokenType::IntegerConst(1), 1),
                Token::new(TokenType::SemiColon, 1),
                Token::new(TokenType::Eof, 1),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Ok(statement!(
                    1,
                    StatementKind::If {
                        condition: Expression::Identifier(identifier!(b"x")),
                        then_branch: Box::new(statement!(
                            1,
                            StatementKind::Return {
                                value: Some(Expression::Integer(1)),
                            }
                        )),
                        else_branch: None,
                    }
                ))
            );
        }

        #[test]
        fn parses_if_with_else() {
            let tokens = vec![
                Token::new(TokenType::If, 1),
                Token::new(TokenType::Lparen, 1),
                Token::new(TokenType::Id(b"x".to_vec()), 1),
                Token::new(TokenType::Rparen, 1),
                Token::new(TokenType::Return, 1),
                Token::new(TokenType::IntegerConst(1), 1),
                Token::new(TokenType::SemiColon, 1),
                Token::new(TokenType::Else, 1),
                Token::new(TokenType::Return, 1),
                Token::new(TokenType::IntegerConst(2), 1),
                Token::new(TokenType::SemiColon, 1),
                Token::new(TokenType::Eof, 1),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Ok(statement!(
                    1,
                    StatementKind::If {
                        condition: Expression::Identifier(identifier!(b"x")),
                        then_branch: Box::new(statement!(
                            1,
                            StatementKind::Return {
                                value: Some(Expression::Integer(1)),
                            }
                        )),
                        else_branch: Some(Box::new(statement!(
                            1,
                            StatementKind::Return {
                                value: Some(Expression::Integer(2)),
                            }
                        ))),
                    }
                ))
            );
        }

        #[test]
        fn if_requires_opening_parenthesis() {
            let tokens = vec![
                Token::new(TokenType::If, 1),
                Token::new(TokenType::Id(b"x".to_vec()), 1),
                Token::new(TokenType::Rparen, 1),
                Token::new(TokenType::Return, 1),
                Token::new(TokenType::IntegerConst(1), 1),
                Token::new(TokenType::SemiColon, 1),
                Token::new(TokenType::Eof, 1),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Err(ParserError::UnexpectedToken {
                    expected: "'('",
                    found: TokenType::Id(b"x".to_vec()),
                    line: 1,
                })
            );
        }

        #[test]
        fn if_requires_closing_parenthesis() {
            let tokens = vec![
                Token::new(TokenType::If, 1),
                Token::new(TokenType::Lparen, 1),
                Token::new(TokenType::Id(b"x".to_vec()), 1),
                Token::new(TokenType::Return, 1),
                Token::new(TokenType::IntegerConst(1), 1),
                Token::new(TokenType::SemiColon, 1),
                Token::new(TokenType::Eof, 1),
            ];

            let mut parser = make_parser(tokens);

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
                Token::new(TokenType::If, 1),
                Token::new(TokenType::Lparen, 1),
                Token::new(TokenType::Id(b"a".to_vec()), 1),
                Token::new(TokenType::Rparen, 1),
                Token::new(TokenType::If, 2),
                Token::new(TokenType::Lparen, 2),
                Token::new(TokenType::Id(b"b".to_vec()), 2),
                Token::new(TokenType::Rparen, 2),
                Token::new(TokenType::Return, 3),
                Token::new(TokenType::IntegerConst(1), 3),
                Token::new(TokenType::SemiColon, 3),
                Token::new(TokenType::Else, 4),
                Token::new(TokenType::Return, 4),
                Token::new(TokenType::IntegerConst(2), 4),
                Token::new(TokenType::SemiColon, 4),
                Token::new(TokenType::Eof, 4),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Ok(statement!(
                    1,
                    StatementKind::If {
                        condition: Expression::Identifier(identifier!(b"a")),
                        then_branch: Box::new(statement!(
                            2,
                            StatementKind::If {
                                condition: Expression::Identifier(identifier!(b"b", 2)),
                                then_branch: Box::new(statement!(
                                    3,
                                    StatementKind::Return {
                                        value: Some(Expression::Integer(1)),
                                    }
                                )),
                                else_branch: Some(Box::new(statement!(
                                    4,
                                    StatementKind::Return {
                                        value: Some(Expression::Integer(2)),
                                    }
                                ))),
                            }
                        )),
                        else_branch: None,
                    }
                ))
            );
        }

        #[test]
        fn if_accepts_empty_statement_as_body() {
            let tokens = vec![
                Token::new(TokenType::If, 1),
                Token::new(TokenType::Lparen, 1),
                Token::new(TokenType::Id(b"x".to_vec()), 1),
                Token::new(TokenType::Rparen, 1),
                Token::new(TokenType::SemiColon, 1),
                Token::new(TokenType::Eof, 1),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Ok(statement!(
                    1,
                    StatementKind::If {
                        condition: Expression::Identifier(identifier!(b"x")),
                        then_branch: Box::new(statement!(1, StatementKind::Empty)),
                        else_branch: None,
                    }
                ))
            );
        }

        #[test]
        fn else_requires_statement() {
            let tokens = vec![
                Token::new(TokenType::If, 1),
                Token::new(TokenType::Lparen, 1),
                Token::new(TokenType::Id(b"x".to_vec()), 1),
                Token::new(TokenType::Rparen, 1),
                Token::new(TokenType::Return, 1),
                Token::new(TokenType::IntegerConst(1), 1),
                Token::new(TokenType::SemiColon, 1),
                Token::new(TokenType::Else, 1),
                Token::new(TokenType::Eof, 1),
            ];

            let mut parser = make_parser(tokens);

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
                Token::new(TokenType::If, 1),
                Token::new(TokenType::Lparen, 1),
                Token::new(TokenType::Id(b"x".to_vec()), 1),
                Token::new(TokenType::Rparen, 1),
                Token::new(TokenType::Return, 1),
                Token::new(TokenType::IntegerConst(1), 1),
                Token::new(TokenType::SemiColon, 1),
                Token::new(TokenType::Id(b"y".to_vec()), 2),
                Token::new(TokenType::Assign, 2),
                Token::new(TokenType::IntegerConst(2), 2),
                Token::new(TokenType::SemiColon, 2),
                Token::new(TokenType::Eof, 2),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Ok(statement!(
                    1,
                    StatementKind::If {
                        condition: Expression::Identifier(identifier!(b"x")),
                        then_branch: Box::new(statement!(
                            1,
                            StatementKind::Return {
                                value: Some(Expression::Integer(1)),
                            }
                        )),
                        else_branch: None,
                    }
                ))
            );

            assert_eq!(
                parser.parse_statement(),
                Ok(statement!(
                    2,
                    StatementKind::Assignment(Assignment {
                        target: LValue::Identifier(identifier!(b"y", 2)),
                        value: Expression::Integer(2),
                    })
                ))
            );
        }
    }
);

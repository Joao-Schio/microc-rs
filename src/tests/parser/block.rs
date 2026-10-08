use crate::{
    ast::{
        expression::Expression,
        statement::{Assignment, Block, LValue, StatementKind},
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
        fn parses_empty_block() {
            let tokens = vec![
                Token::new(TokenType::LBrace, 1),
                Token::new(TokenType::RBrace, 1),
                Token::new(TokenType::Eof, 1),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Ok(statement!(
                    1,
                    StatementKind::Block(Block {
                        declarations: vec![],
                        statements: vec![],
                    })
                ))
            );
        }

        #[test]
        fn parses_block_with_multiple_statements() {
            let tokens = vec![
                Token::new(TokenType::LBrace, 1),
                Token::new(TokenType::Id(b"x".to_vec()), 2),
                Token::new(TokenType::Assign, 2),
                Token::new(TokenType::IntegerConst(10), 2),
                Token::new(TokenType::SemiColon, 2),
                Token::new(TokenType::Return, 3),
                Token::new(TokenType::IntegerConst(0), 3),
                Token::new(TokenType::SemiColon, 3),
                Token::new(TokenType::RBrace, 4),
                Token::new(TokenType::Eof, 4),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Ok(statement!(
                    1,
                    StatementKind::Block(Block {
                        declarations: vec![],
                        statements: vec![
                            statement!(
                                2,
                                StatementKind::Assignment(Assignment {
                                    target: LValue::Identifier(identifier!(b"x", 2)),
                                    value: Expression::Integer(10),
                                })
                            ),
                            statement!(
                                3,
                                StatementKind::Return {
                                    value: Some(Expression::Integer(0)),
                                }
                            ),
                        ],
                    })
                ))
            );
        }

        #[test]
        fn parses_nested_blocks() {
            let tokens = vec![
                Token::new(TokenType::LBrace, 1),
                Token::new(TokenType::LBrace, 2),
                Token::new(TokenType::Return, 3),
                Token::new(TokenType::IntegerConst(1), 3),
                Token::new(TokenType::SemiColon, 3),
                Token::new(TokenType::RBrace, 4),
                Token::new(TokenType::RBrace, 5),
                Token::new(TokenType::Eof, 5),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Ok(statement!(
                    1,
                    StatementKind::Block(Block {
                        declarations: vec![],
                        statements: vec![statement!(
                            2,
                            StatementKind::Block(Block {
                                declarations: vec![],
                                statements: vec![statement!(
                                    3,
                                    StatementKind::Return {
                                        value: Some(Expression::Integer(1)),
                                    }
                                )],
                            })
                        )],
                    })
                ))
            );
        }

        #[test]
        fn block_consumes_only_its_own_tokens() {
            let tokens = vec![
                Token::new(TokenType::LBrace, 1),
                Token::new(TokenType::RBrace, 1),
                Token::new(TokenType::Return, 2),
                Token::new(TokenType::IntegerConst(1), 2),
                Token::new(TokenType::SemiColon, 2),
                Token::new(TokenType::Eof, 2),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Ok(statement!(
                    1,
                    StatementKind::Block(Block {
                        declarations: vec![],
                        statements: vec![],
                    })
                ))
            );
            assert_eq!(
                parser.parse_statement(),
                Ok(statement!(
                    2,
                    StatementKind::Return {
                        value: Some(Expression::Integer(1)),
                    }
                ))
            );
        }

        #[test]
        fn parses_if_with_block_branches() {
            let tokens = vec![
                Token::new(TokenType::If, 1),
                Token::new(TokenType::Lparen, 1),
                Token::new(TokenType::IntegerConst(1), 1),
                Token::new(TokenType::Rparen, 1),
                Token::new(TokenType::LBrace, 1),
                Token::new(TokenType::Return, 2),
                Token::new(TokenType::IntegerConst(1), 2),
                Token::new(TokenType::SemiColon, 2),
                Token::new(TokenType::RBrace, 3),
                Token::new(TokenType::Else, 3),
                Token::new(TokenType::LBrace, 3),
                Token::new(TokenType::Return, 4),
                Token::new(TokenType::IntegerConst(0), 4),
                Token::new(TokenType::SemiColon, 4),
                Token::new(TokenType::RBrace, 5),
                Token::new(TokenType::Eof, 5),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Ok(statement!(
                    1,
                    StatementKind::If {
                        condition: Expression::Integer(1),
                        then_branch: Box::new(statement!(
                            1,
                            StatementKind::Block(Block {
                                declarations: vec![],
                                statements: vec![statement!(
                                    2,
                                    StatementKind::Return {
                                        value: Some(Expression::Integer(1)),
                                    }
                                )],
                            })
                        )),
                        else_branch: Some(Box::new(statement!(
                            3,
                            StatementKind::Block(Block {
                                declarations: vec![],
                                statements: vec![statement!(
                                    4,
                                    StatementKind::Return {
                                        value: Some(Expression::Integer(0)),
                                    }
                                )],
                            })
                        ))),
                    }
                ))
            );
        }

        #[test]
        fn unterminated_block_reports_missing_closing_brace() {
            let tokens = vec![
                Token::new(TokenType::LBrace, 1),
                Token::new(TokenType::Return, 2),
                Token::new(TokenType::SemiColon, 2),
                Token::new(TokenType::Eof, 2),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Err(ParserError::UnexpectedToken {
                    expected: "'}'",
                    found: TokenType::Eof,
                    line: 2,
                })
            );
        }
    }
);

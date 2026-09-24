use crate::{
    ast::{
        expression::{BinaryOp, Expression},
        statement::{Assignment, Block, LValue, Statement},
    },
    token::{Token, TokenType},
};

parser_contract!(
    default_statement_parser,
    crate::parser::ExprParser,
    crate::parser::StatementParser,
    {
        #[test]
        fn parses_for_statement() {
            let tokens = vec![
                Token::new(TokenType::For, 1),
                Token::new(TokenType::Lparen, 1),
                // i = 0
                Token::new(TokenType::Id(b"i".to_vec()), 1),
                Token::new(TokenType::Assign, 1),
                Token::new(TokenType::IntegerConst(0), 1),
                Token::new(TokenType::SemiColon, 1),
                // i < 10
                Token::new(TokenType::Id(b"i".to_vec()), 1),
                Token::new(TokenType::Lt, 1),
                Token::new(TokenType::IntegerConst(10), 1),
                Token::new(TokenType::SemiColon, 1),
                // i = i + 1
                Token::new(TokenType::Id(b"i".to_vec()), 1),
                Token::new(TokenType::Assign, 1),
                Token::new(TokenType::Id(b"i".to_vec()), 1),
                Token::new(TokenType::Plus, 1),
                Token::new(TokenType::IntegerConst(1), 1),
                Token::new(TokenType::Rparen, 1),
                Token::new(TokenType::SemiColon, 1),
                Token::new(TokenType::Eof, 1),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Ok(Statement::For {
                    initialization: Assignment {
                        target: LValue::Identifier(b"i".to_vec()),
                        value: Expression::Integer(0),
                    },
                    condition: Expression::Binary {
                        left: Box::new(Expression::Identifier(b"i".to_vec())),
                        op: BinaryOp::Less,
                        right: Box::new(Expression::Integer(10)),
                    },
                    update: Assignment {
                        target: LValue::Identifier(b"i".to_vec()),
                        value: Expression::Binary {
                            left: Box::new(Expression::Identifier(b"i".to_vec())),
                            op: BinaryOp::Add,
                            right: Box::new(Expression::Integer(1)),
                        },
                    },
                    body: Box::new(Statement::Empty),
                })
            );
        }

        #[test]
        fn parses_for_with_block_body() {
            let tokens = vec![
                Token::new(TokenType::For, 1),
                Token::new(TokenType::Lparen, 1),
                Token::new(TokenType::Id(b"i".to_vec()), 1),
                Token::new(TokenType::Assign, 1),
                Token::new(TokenType::IntegerConst(0), 1),
                Token::new(TokenType::SemiColon, 1),
                Token::new(TokenType::Id(b"i".to_vec()), 1),
                Token::new(TokenType::Lt, 1),
                Token::new(TokenType::IntegerConst(10), 1),
                Token::new(TokenType::SemiColon, 1),
                Token::new(TokenType::Id(b"i".to_vec()), 1),
                Token::new(TokenType::Assign, 1),
                Token::new(TokenType::Id(b"i".to_vec()), 1),
                Token::new(TokenType::Plus, 1),
                Token::new(TokenType::IntegerConst(1), 1),
                Token::new(TokenType::Rparen, 1),
                Token::new(TokenType::LBrace, 1),
                Token::new(TokenType::Return, 2),
                Token::new(TokenType::Id(b"i".to_vec()), 2),
                Token::new(TokenType::SemiColon, 2),
                Token::new(TokenType::RBrace, 3),
                Token::new(TokenType::Eof, 3),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Ok(Statement::For {
                    initialization: Assignment {
                        target: LValue::Identifier(b"i".to_vec()),
                        value: Expression::Integer(0),
                    },
                    condition: Expression::Binary {
                        left: Box::new(Expression::Identifier(b"i".to_vec())),
                        op: BinaryOp::Less,
                        right: Box::new(Expression::Integer(10)),
                    },
                    update: Assignment {
                        target: LValue::Identifier(b"i".to_vec()),
                        value: Expression::Binary {
                            left: Box::new(Expression::Identifier(b"i".to_vec())),
                            op: BinaryOp::Add,
                            right: Box::new(Expression::Integer(1)),
                        },
                    },
                    body: Box::new(Statement::Block(Block {
                        declarations: vec![],
                        statements: vec![Statement::Return {
                            value: Some(Expression::Identifier(b"i".to_vec())),
                        },],
                    })),
                })
            );
        }
    }
);

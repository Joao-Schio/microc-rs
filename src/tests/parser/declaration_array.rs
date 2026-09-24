use crate::{
    ast::statement::{Block, Statement, Type, VariableDeclaration},
    parser::ParserError,
    token::{Token, TokenType},
};

parser_contract!(
    default_statement_parser,
    crate::parser::ExprParser,
    crate::parser::StatementParser,
    {
        #[test]
        fn parses_char_array_declaration() {
            let tokens = vec![
                Token::new(TokenType::LBrace, 1),
                Token::new(TokenType::Char, 2),
                Token::new(TokenType::Id(b"buffer".to_vec()), 2),
                Token::new(TokenType::LBracket, 2),
                Token::new(TokenType::IntegerConst(32), 2),
                Token::new(TokenType::RBracket, 2),
                Token::new(TokenType::SemiColon, 2),
                Token::new(TokenType::RBrace, 3),
                Token::new(TokenType::Eof, 3),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Ok(Statement::Block(Block {
                    declarations: vec![VariableDeclaration::Array {
                        data_type: Type::Char,
                        name: b"buffer".to_vec(),
                        length: 32,
                    }],
                    statements: vec![],
                }))
            );
        }

        #[test]
        fn preserves_order_for_mixed_scalar_and_array_declarations() {
            let tokens = vec![
                Token::new(TokenType::LBrace, 1),
                Token::new(TokenType::Int, 2),
                Token::new(TokenType::Id(b"count".to_vec()), 2),
                Token::new(TokenType::SemiColon, 2),
                Token::new(TokenType::Char, 3),
                Token::new(TokenType::Id(b"buffer".to_vec()), 3),
                Token::new(TokenType::LBracket, 3),
                Token::new(TokenType::IntegerConst(8), 3),
                Token::new(TokenType::RBracket, 3),
                Token::new(TokenType::SemiColon, 3),
                Token::new(TokenType::Int, 4),
                Token::new(TokenType::Id(b"values".to_vec()), 4),
                Token::new(TokenType::LBracket, 4),
                Token::new(TokenType::IntegerConst(4), 4),
                Token::new(TokenType::RBracket, 4),
                Token::new(TokenType::SemiColon, 4),
                Token::new(TokenType::RBrace, 5),
                Token::new(TokenType::Eof, 5),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Ok(Statement::Block(Block {
                    declarations: vec![
                        VariableDeclaration::Scalar {
                            data_type: Type::Int,
                            name: b"count".to_vec(),
                        },
                        VariableDeclaration::Array {
                            data_type: Type::Char,
                            name: b"buffer".to_vec(),
                            length: 8,
                        },
                        VariableDeclaration::Array {
                            data_type: Type::Int,
                            name: b"values".to_vec(),
                            length: 4,
                        },
                    ],
                    statements: vec![],
                }))
            );
        }

        #[test]
        fn array_declaration_requires_integer_length() {
            let tokens = vec![
                Token::new(TokenType::LBrace, 1),
                Token::new(TokenType::Int, 2),
                Token::new(TokenType::Id(b"values".to_vec()), 2),
                Token::new(TokenType::LBracket, 2),
                Token::new(TokenType::RBracket, 2),
                Token::new(TokenType::SemiColon, 2),
                Token::new(TokenType::RBrace, 3),
                Token::new(TokenType::Eof, 3),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Err(ParserError::UnexpectedToken {
                    expected: "IntegerConst",
                    found: TokenType::RBracket,
                    line: 2,
                })
            );
        }

        #[test]
        fn array_declaration_requires_closing_bracket() {
            let tokens = vec![
                Token::new(TokenType::LBrace, 1),
                Token::new(TokenType::Int, 2),
                Token::new(TokenType::Id(b"values".to_vec()), 2),
                Token::new(TokenType::LBracket, 2),
                Token::new(TokenType::IntegerConst(10), 2),
                Token::new(TokenType::SemiColon, 2),
                Token::new(TokenType::RBrace, 3),
                Token::new(TokenType::Eof, 3),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Err(ParserError::UnexpectedToken {
                    expected: "']'",
                    found: TokenType::SemiColon,
                    line: 2,
                })
            );
        }

        #[test]
        fn array_declaration_requires_semicolon() {
            let tokens = vec![
                Token::new(TokenType::LBrace, 1),
                Token::new(TokenType::Int, 2),
                Token::new(TokenType::Id(b"values".to_vec()), 2),
                Token::new(TokenType::LBracket, 2),
                Token::new(TokenType::IntegerConst(10), 2),
                Token::new(TokenType::RBracket, 2),
                Token::new(TokenType::RBrace, 3),
                Token::new(TokenType::Eof, 3),
            ];

            let mut parser = make_parser(tokens);

            assert_eq!(
                parser.parse_statement(),
                Err(ParserError::UnexpectedToken {
                    expected: "';'",
                    found: TokenType::RBrace,
                    line: 3,
                })
            );
        }
    }
);

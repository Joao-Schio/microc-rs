use crate::token::TokenType;

#[test]
fn maps_every_keyword_to_its_token_type() {
    let cases: &[(&[u8], TokenType)] = &[
        (b"main", TokenType::Main),
        (b"if", TokenType::If),
        (b"else", TokenType::Else),
        (b"for", TokenType::For),
        (b"return", TokenType::Return),
        (b"int", TokenType::Int),
        (b"char", TokenType::Char),
        (b"print", TokenType::Print),
    ];

    for (lexeme, expected) in cases {
        assert_eq!(TokenType::from_keyword(lexeme), Some(expected.clone()));
    }
}

#[test]
fn rejects_non_keywords() {
    let cases: &[&[u8]] = &[b"", b"Main", b"ifx", b"integer", b"returning", b"printf"];

    for lexeme in cases {
        assert_eq!(TokenType::from_keyword(lexeme), None);
    }
}

#[test]
fn fixed_tokens_expose_their_expected_lexeme() {
    let cases = [
        (TokenType::Eof, "end of file"),
        (TokenType::Plus, "'+'"),
        (TokenType::Minus, "'-'"),
        (TokenType::Mul, "'*'"),
        (TokenType::Div, "'/'"),
        (TokenType::Mod, "'%'"),
        (TokenType::Eq, "'=='"),
        (TokenType::Neq, "'!='"),
        (TokenType::Lt, "'<'"),
        (TokenType::Gt, "'>'"),
        (TokenType::Leq, "'<='"),
        (TokenType::Geq, "'>='"),
        (TokenType::And, "'&&'"),
        (TokenType::Or, "'||'"),
        (TokenType::Not, "'!'"),
        (TokenType::Assign, "'='"),
        (TokenType::SemiColon, "';'"),
        (TokenType::Comma, "','"),
        (TokenType::Lparen, "'('"),
        (TokenType::Rparen, "')'"),
        (TokenType::LBrace, "'{'"),
        (TokenType::RBrace, "'}'"),
        (TokenType::LBracket, "'['"),
        (TokenType::RBracket, "']'"),
        (TokenType::Main, "'main'"),
        (TokenType::If, "'if'"),
        (TokenType::Else, "'else'"),
        (TokenType::For, "'for'"),
        (TokenType::Return, "'return'"),
        (TokenType::Int, "'int'"),
        (TokenType::Char, "'char'"),
        (TokenType::Print, "'print'"),
    ];

    for (token_type, expected) in cases {
        assert_eq!(token_type.get_expected_lexeme(), Some(expected));
    }
}

#[test]
fn payload_tokens_do_not_have_fixed_expected_lexemes() {
    let token_types = [
        TokenType::IntegerConst(1),
        TokenType::CharConst(b'a'),
        TokenType::StringConst(b"hello".to_vec()),
        TokenType::Id(b"value".to_vec()),
    ];

    for token_type in token_types {
        assert_eq!(token_type.get_expected_lexeme(), None);
    }
}

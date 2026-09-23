const MAIN_LEXEME: &[u8] = b"main";
const IF_LEXEME: &[u8] = b"if";
const ELSE_LEXEME: &[u8] = b"else";
const FOR_LEXEME: &[u8] = b"for";
const RETURN_LEXEME: &[u8] = b"return";
const INT_LEXEME: &[u8] = b"int";
const CHAR_LEXEME: &[u8] = b"char";
const PRINT_LEXEME: &[u8] = b"print";

const fn bytes_equal(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }

    let mut index = 0;
    while index < left.len() {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }

    true
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenType {
    IntegerConst(i64),
    CharConst(u8),
    StringConst(Vec<u8>),
    Id(Vec<u8>),
    Eof,
    Plus,
    Minus,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Gt,
    Leq,
    Geq,
    And,
    Or,
    Not,
    Assign,
    SemiColon,
    Comma,
    Lparen,
    Rparen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Main,
    If,
    Else,
    For,
    Return,
    Int,
    Char,
    Print,
}

impl TokenType {
    pub(crate) const fn from_keyword(identifier: &[u8]) -> Option<Self> {
        if bytes_equal(identifier, MAIN_LEXEME) {
            Some(Self::Main)
        } else if bytes_equal(identifier, IF_LEXEME) {
            Some(Self::If)
        } else if bytes_equal(identifier, ELSE_LEXEME) {
            Some(Self::Else)
        } else if bytes_equal(identifier, FOR_LEXEME) {
            Some(Self::For)
        } else if bytes_equal(identifier, RETURN_LEXEME) {
            Some(Self::Return)
        } else if bytes_equal(identifier, INT_LEXEME) {
            Some(Self::Int)
        } else if bytes_equal(identifier, CHAR_LEXEME) {
            Some(Self::Char)
        } else if bytes_equal(identifier, PRINT_LEXEME) {
            Some(Self::Print)
        } else {
            None
        }
    }

    const fn fixed_lexeme(&self) -> Option<&'static [u8]> {
        match self {
            Self::CharConst(_) => None,
            Self::Id(_) => None,
            Self::StringConst(_) => None,
            Self::IntegerConst(_) => None,
            Self::And => Some(b"&&"),
            Self::Assign => Some(b"="),
            Self::Char => Some(CHAR_LEXEME),
            Self::Comma => Some(b","),
            Self::Div => Some(b"/"),
            Self::Else => Some(ELSE_LEXEME),
            Self::Eof => Some(b""),
            Self::Eq => Some(b"=="),
            Self::For => Some(FOR_LEXEME),
            Self::Geq => Some(b">="),
            Self::Gt => Some(b">"),
            Self::If => Some(IF_LEXEME),
            Self::LBrace => Some(b"{"),
            Self::LBracket => Some(b"["),
            Self::Leq => Some(b"<="),
            Self::Lparen => Some(b"("),
            Self::Lt => Some(b"<"),
            Self::Main => Some(MAIN_LEXEME),
            Self::Minus => Some(b"-"),
            Self::Mod => Some(b"%"),
            Self::Mul => Some(b"*"),
            Self::Neq => Some(b"!="),
            Self::Not => Some(b"!"),
            Self::Or => Some(b"||"),
            Self::Plus => Some(b"+"),
            Self::Print => Some(PRINT_LEXEME),
            Self::RBrace => Some(b"}"),
            Self::RBracket => Some(b"]"),
            Self::Return => Some(RETURN_LEXEME),
            Self::Rparen => Some(b")"),
            Self::SemiColon => Some(b";"),
            Self::Int => Some(INT_LEXEME),
        }
    }
}

#[derive(Debug)]
pub struct Token {
    tipo: TokenType,
    linha: usize,
}

impl Token {
    pub const fn new(tipo: TokenType, linha: usize) -> Self {
        Self { tipo, linha }
    }

    pub const fn get_tok_type(&self) -> &TokenType {
        &self.tipo
    }

    pub const fn get_linha(&self) -> usize {
        self.linha
    }

    pub const fn into_type(self) -> TokenType {
        self.tipo
    }

    pub fn into_lexeme(self) -> Vec<u8> {
        if let Some(value) = self.tipo.fixed_lexeme() {
            return value.to_vec();
        }

        match self.tipo {
            TokenType::CharConst(character) => vec![character],
            TokenType::StringConst(string) => string,
            TokenType::Id(identifier) => identifier,
            TokenType::IntegerConst(integer) => integer.to_string().into_bytes(),
            _ => unreachable!("fixed token types must have a fixed lexeme"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Token, TokenType};

    const IF_TOKEN: Option<TokenType> = TokenType::from_keyword(b"if");
    const NOT_A_KEYWORD: Option<TokenType> = TokenType::from_keyword(b"ifx");
    const PLUS_LEXEME: Option<&[u8]> = TokenType::Plus.fixed_lexeme();
    const TOKEN_LINE: usize = Token::new(TokenType::Plus, 7).get_linha();

    #[test]
    fn keyword_lookup_is_const_evaluable() {
        assert_eq!(IF_TOKEN, Some(TokenType::If));
        assert_eq!(NOT_A_KEYWORD, None);
    }

    #[test]
    fn fixed_lexeme_is_const_evaluable() {
        assert_eq!(PLUS_LEXEME, Some(b"+".as_slice()));
    }

    #[test]
    fn token_metadata_is_const_evaluable() {
        assert_eq!(TOKEN_LINE, 7);
    }
}

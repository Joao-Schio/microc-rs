const MAIN_KEYWORD: &[u8] = b"main";
const IF_KEYWORD: &[u8] = b"if";
const ELSE_KEYWORD: &[u8] = b"else";
const FOR_KEYWORD: &[u8] = b"for";
const RETURN_KEYWORD: &[u8] = b"return";
const INT_KEYWORD: &[u8] = b"int";
const CHAR_KEYWORD: &[u8] = b"char";
const PRINT_KEYWORD: &[u8] = b"print";

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
        if bytes_equal(identifier, MAIN_KEYWORD) {
            Some(Self::Main)
        } else if bytes_equal(identifier, IF_KEYWORD) {
            Some(Self::If)
        } else if bytes_equal(identifier, ELSE_KEYWORD) {
            Some(Self::Else)
        } else if bytes_equal(identifier, FOR_KEYWORD) {
            Some(Self::For)
        } else if bytes_equal(identifier, RETURN_KEYWORD) {
            Some(Self::Return)
        } else if bytes_equal(identifier, INT_KEYWORD) {
            Some(Self::Int)
        } else if bytes_equal(identifier, CHAR_KEYWORD) {
            Some(Self::Char)
        } else if bytes_equal(identifier, PRINT_KEYWORD) {
            Some(Self::Print)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Token {
    token_type: TokenType,
    line: usize,
}

impl Token {
    pub const fn new(token_type: TokenType, line: usize) -> Self {
        Self { token_type, line }
    }

    pub const fn token_type(&self) -> &TokenType {
        &self.token_type
    }

    pub const fn line(&self) -> usize {
        self.line
    }

    pub fn into_type(self) -> TokenType {
        self.token_type
    }
}

#[cfg(test)]
mod tests {
    use super::{Token, TokenType};

    const IF_TOKEN: Option<TokenType> = TokenType::from_keyword(b"if");
    const NOT_A_KEYWORD: Option<TokenType> = TokenType::from_keyword(b"ifx");
    const TOKEN: Token = Token::new(TokenType::Plus, 7);

    #[test]
    fn keyword_lookup_is_const_evaluable() {
        assert_eq!(IF_TOKEN, Some(TokenType::If));
        assert_eq!(NOT_A_KEYWORD, None);
    }

    #[test]
    fn token_constructor_is_const_evaluable() {
        assert_eq!(TOKEN.line(), 7);
        assert_eq!(TOKEN.token_type(), &TokenType::Plus);
    }
}

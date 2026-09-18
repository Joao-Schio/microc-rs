#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TokenType {
    IntegerConst(i64),
    CharConst,
    StringConst,
    Id,
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

#[derive(Debug)]
pub struct Token {
    tipo: TokenType,
    linha: usize,
    lexema: Vec<u8>,
}

impl Token {
    pub fn new(tipo: TokenType, linha: usize, lexema: Vec<u8>) -> Self {
        Self {
            tipo,
            linha,
            lexema,
        }
    }

    pub fn get_tok_type(&self) -> &TokenType {
        &self.tipo
    }

    pub fn get_lexema(&self) -> &[u8] {
        &self.lexema
    }

    pub fn get_linha(&self) -> usize {
        self.linha
    }
}

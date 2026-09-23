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
    const fn get_lexema(&self) -> Option<&'static str> {
        match self {
            Self::CharConst(_) => None,
            Self::Id(_) => None,
            Self::StringConst(_) => None,
            Self::IntegerConst(_) => None,
            Self::And => Some("&&"),
            Self::Assign => Some("="),
            Self::Char => Some("char"),
            Self::Comma => Some(","),
            Self::Div => Some("/"),
            Self::Else => Some("else"),
            Self::Eof => Some(""),
            Self::Eq => Some("=="),
            Self::For => Some("for"),
            Self::Geq => Some(">="),
            Self::Gt => Some(">"),
            Self::If => Some("if"),
            Self::LBrace => Some("{"),
            Self::LBracket => Some("["),
            Self::Leq => Some("<="),
            Self::Lparen => Some("("),
            Self::Lt => Some("<"),
            Self::Main => Some("main"),
            Self::Minus => Some("-"),
            Self::Mod => Some("%"),
            Self::Mul => Some("*"),
            Self::Neq => Some("!="),
            Self::Not => Some("!"),
            Self::Or => Some("||"),
            Self::Plus => Some("+"),
            Self::Print => Some("print"),
            Self::RBrace => Some("}"),
            Self::RBracket => Some("]"),
            Self::Return => Some("return"),
            Self::Rparen => Some(")"),
            Self::SemiColon => Some(";"),
            Self::Int => Some("int"),
        }
    }
}

#[derive(Debug)]
pub struct Token {
    tipo: TokenType,
    linha: usize,
}

impl Token {
    pub fn new(tipo: TokenType, linha: usize) -> Self {
        Self { tipo, linha }
    }

    pub fn get_tok_type(&self) -> &TokenType {
        &self.tipo
    }

    pub fn get_linha(&self) -> usize {
        self.linha
    }

    pub fn into_lexeme(self) -> Vec<u8> {
        if let Some(val) = self.tipo.get_lexema() {
            return val.to_owned().into_bytes();
        }
        match self.tipo {
            TokenType::CharConst(c) => vec![c],
            TokenType::StringConst(str_const) => str_const,
            TokenType::Id(id) => id,
            TokenType::IntegerConst(integer) => integer.to_string().into_bytes(),
            _ => b"unreachable".to_vec(),
        }
    }
}

use crate::{
    lexer::{Lexer, LexerError, TLexer},
    scanner::{ScannerError, TScanner},
    token::TokenType,
};

pub(super) struct DummyScanner {
    input: Vec<u8>,
    position: usize,
    line: usize,
    column: usize,
}

impl DummyScanner {
    pub(super) fn new(input: &str) -> Self {
        Self {
            input: input.as_bytes().to_vec(),
            position: 0,
            line: 1,
            column: 1,
        }
    }
}

impl TScanner for DummyScanner {
    fn get_line(&self) -> usize {
        self.line
    }

    fn get_next(&mut self) -> Result<Option<u8>, ScannerError> {
        let value = self.peek_next();

        if let Some(byte) = value {
            self.position += 1;

            if byte == b'\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }

        Ok(value)
    }

    fn peek_next(&self) -> Option<u8> {
        self.input.get(self.position).copied()
    }

    fn get_column(&self) -> usize {
        self.column
    }
}

pub(super) struct FailingScanner;

impl TScanner for FailingScanner {
    fn peek_next(&self) -> Option<u8> {
        None
    }

    fn get_next(&mut self) -> Result<Option<u8>, ScannerError> {
        Err(ScannerError::Io(std::io::Error::other(
            "test scanner failure",
        )))
    }

    fn get_line(&self) -> usize {
        1
    }

    fn get_column(&self) -> usize {
        1
    }
}

pub(super) fn make_lexer(input: &str) -> Lexer<DummyScanner> {
    Lexer::new(DummyScanner::new(input))
}

pub(super) fn make_failing_lexer() -> Lexer<FailingScanner> {
    Lexer::new(FailingScanner)
}

fn canonical_spelling(token_type: &TokenType) -> Vec<u8> {
    match token_type {
        TokenType::IntegerConst(value) => value.to_string().into_bytes(),
        TokenType::CharConst(value) => vec![*value],
        TokenType::StringConst(value) | TokenType::Id(value) => value.clone(),
        TokenType::Eof => Vec::new(),
        TokenType::Plus => b"+".to_vec(),
        TokenType::Minus => b"-".to_vec(),
        TokenType::Mul => b"*".to_vec(),
        TokenType::Div => b"/".to_vec(),
        TokenType::Mod => b"%".to_vec(),
        TokenType::Eq => b"==".to_vec(),
        TokenType::Neq => b"!=".to_vec(),
        TokenType::Lt => b"<".to_vec(),
        TokenType::Gt => b">".to_vec(),
        TokenType::Leq => b"<=".to_vec(),
        TokenType::Geq => b">=".to_vec(),
        TokenType::And => b"&&".to_vec(),
        TokenType::Or => b"||".to_vec(),
        TokenType::Not => b"!".to_vec(),
        TokenType::Assign => b"=".to_vec(),
        TokenType::SemiColon => b";".to_vec(),
        TokenType::Comma => b",".to_vec(),
        TokenType::Lparen => b"(".to_vec(),
        TokenType::Rparen => b")".to_vec(),
        TokenType::LBrace => b"{".to_vec(),
        TokenType::RBrace => b"}".to_vec(),
        TokenType::LBracket => b"[".to_vec(),
        TokenType::RBracket => b"]".to_vec(),
        TokenType::Main => b"main".to_vec(),
        TokenType::If => b"if".to_vec(),
        TokenType::Else => b"else".to_vec(),
        TokenType::For => b"for".to_vec(),
        TokenType::Return => b"return".to_vec(),
        TokenType::Int => b"int".to_vec(),
        TokenType::Char => b"char".to_vec(),
        TokenType::Print => b"print".to_vec(),
    }
}

pub(super) fn assert_token<L: TLexer>(
    lexer: &mut L,
    expected_type: TokenType,
    expected_spelling: &str,
) {
    let token = lexer
        .get_prox_token()
        .unwrap_or_else(|error| panic!("expected token, got lexer error: {error:?}"));

    assert_eq!(token.token_type(), &expected_type);
    assert_eq!(
        canonical_spelling(token.token_type()).as_slice(),
        expected_spelling.as_bytes()
    );
}

pub(super) fn assert_lexer_error<L: TLexer>(lexer: &mut L) -> LexerError {
    match lexer.get_prox_token() {
        Err(error) => error,
        Ok(_) => panic!("expected lexer error, got a token"),
    }
}

use std::collections::HashMap;

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
    Lexer::new(DummyScanner::new(input), HashMap::new())
}

pub(super) fn make_lexer_with_reserved_words(
    input: &str,
    reserved_words: HashMap<&'static str, TokenType>,
) -> Lexer<DummyScanner> {
    let reserved_words: HashMap<&'static [u8], TokenType> = reserved_words
        .into_iter()
        .map(|(word, token_type)| (word.as_bytes(), token_type))
        .collect();

    Lexer::new(DummyScanner::new(input), reserved_words)
}

pub(super) fn make_failing_lexer() -> Lexer<FailingScanner> {
    Lexer::new(FailingScanner, HashMap::new())
}

pub(super) fn assert_token<L: TLexer>(
    lexer: &mut L,
    expected_type: TokenType,
    expected_lexeme: &str,
) {
    let token = lexer
        .get_prox_token()
        .unwrap_or_else(|error| panic!("expected token, got lexer error: {error:?}"));

    assert_eq!(token.get_tok_type(), &expected_type);

    let lexeme = token.into_lexeme();
    assert_eq!(lexeme.as_slice(), expected_lexeme.as_bytes());
}

pub(super) fn assert_lexer_error<L: TLexer>(lexer: &mut L) -> LexerError {
    match lexer.get_prox_token() {
        Err(error) => error,
        Ok(_) => panic!("expected lexer error, got a token"),
    }
}

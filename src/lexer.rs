use std::{error::Error, fmt};

use crate::{
    scanner::{ScannerError, TScanner},
    token::{Token, TokenType},
};

pub fn tokenize<L: TLexer>(lexer: &mut L) -> Result<Vec<Token>, LexerError> {
    let mut tokens = Vec::new();

    loop {
        let token = lexer.get_prox_token()?;
        let is_eof = matches!(token.get_tok_type(), TokenType::Eof);

        tokens.push(token);

        if is_eof {
            return Ok(tokens);
        }
    }
}

#[derive(Debug)]
pub enum LexerError {
    Scanner(ScannerError),

    UnexpectedCharacter {
        character: u8,
        line: usize,
        column: usize,
    },

    InvalidCharacterLiteral {
        line: usize,
        column: usize,
    },

    UnterminatedString {
        line: usize,
        column: usize,
    },

    InvalidLogicalOperator {
        character: u8,
        line: usize,
        column: usize,
    },

    IntegerOutOfRange {
        lexeme: String,
        line: usize,
        column: usize,
    },
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scanner(error) => write!(f, "{error}"),
            Self::UnexpectedCharacter {
                character,
                line,
                column,
            } => write!(
                f,
                "unexpected character '{}' at line {line}, column {column}",
                *character as char
            ),
            Self::InvalidCharacterLiteral { line, column } => {
                write!(
                    f,
                    "invalid character literal at line {line}, column {column}"
                )
            }
            Self::UnterminatedString { line, column } => {
                write!(f, "unterminated string at line {line}, column {column}")
            }
            Self::InvalidLogicalOperator {
                character,
                line,
                column,
            } => write!(
                f,
                "invalid logical operator '{}' at line {line}, column {column}",
                *character as char
            ),
            Self::IntegerOutOfRange {
                lexeme,
                line,
                column,
            } => write!(
                f,
                "integer literal '{lexeme}' is out of range at line {line}, column {column}"
            ),
        }
    }
}

impl Error for LexerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Scanner(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ScannerError> for LexerError {
    fn from(error: ScannerError) -> Self {
        Self::Scanner(error)
    }
}

pub trait TLexer {
    fn get_prox_token(&mut self) -> Result<Token, LexerError>;
}

pub struct Lexer<S: TScanner> {
    scanner: S,
}

impl<S: TScanner> Lexer<S> {
    pub const fn new(scanner: S) -> Self {
        Self { scanner }
    }

    fn token_start_location(&self) -> (usize, usize) {
        (
            self.scanner.get_line(),
            self.scanner.get_column().saturating_sub(1),
        )
    }

    fn discard_comment(&mut self) -> Result<(), LexerError> {
        while let Some(c) = self.scanner.peek_next() {
            self.discard_next()?;
            if c == b'\n' {
                break;
            }
        }

        Ok(())
    }

    fn get_next_meaningful_char(&mut self) -> Result<Option<u8>, LexerError> {
        loop {
            let Some(initial) = self.scanner.get_next()? else {
                return Ok(None);
            };

            if initial.is_ascii_whitespace() {
                continue;
            }

            if initial == b'/' && self.scanner.peek_next() == Some(b'/') {
                self.discard_next()?;
                self.discard_comment()?;
                continue;
            }

            return Ok(Some(initial));
        }
    }

    fn lex_token(&mut self, initial: u8) -> Result<Token, LexerError> {
        match initial {
            b'+' => Ok(self.single_char_token(TokenType::Plus)),
            b'-' => Ok(self.single_char_token(TokenType::Minus)),
            b'*' => Ok(self.single_char_token(TokenType::Mul)),
            b'%' => Ok(self.single_char_token(TokenType::Mod)),
            b'/' => Ok(self.single_char_token(TokenType::Div)),
            b'=' => self.match_equal(),
            b'>' => self.match_greater(),
            b'<' => self.match_lesser(),
            b'&' => self.match_and(),
            b'|' => self.match_or(),
            b'!' => self.match_not(),
            b'"' => self.match_quotes(),
            b'\'' => self.match_single_quote(),
            b',' => Ok(self.single_char_token(TokenType::Comma)),
            b';' => Ok(self.single_char_token(TokenType::SemiColon)),
            b'(' => Ok(self.single_char_token(TokenType::Lparen)),
            b')' => Ok(self.single_char_token(TokenType::Rparen)),
            b'{' => Ok(self.single_char_token(TokenType::LBrace)),
            b'}' => Ok(self.single_char_token(TokenType::RBrace)),
            b'[' => Ok(self.single_char_token(TokenType::LBracket)),
            b']' => Ok(self.single_char_token(TokenType::RBracket)),
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.match_letters_tokens(initial),
            b'0'..=b'9' => self.match_numeric(initial),
            character => {
                let (line, column) = self.token_start_location();
                Err(LexerError::UnexpectedCharacter {
                    character,
                    line,
                    column,
                })
            }
        }
    }

    fn is_allowed_identifier_character(character: u8) -> bool {
        character.is_ascii_alphanumeric() || character == b'_'
    }

    fn is_allowed_numeric_character(character: u8) -> bool {
        character.is_ascii_digit()
    }

    fn match_letters_tokens(&mut self, initial: u8) -> Result<Token, LexerError> {
        let mut identifier = vec![initial];

        while let Some(next) = self.scanner.peek_next() {
            if !Self::is_allowed_identifier_character(next) {
                break;
            }

            self.discard_next()?;
            identifier.push(next);
        }

        let token_type = match TokenType::from_keyword(&identifier) {
            Some(keyword) => keyword,
            None => TokenType::Id(identifier),
        };

        Ok(Token::new(token_type, self.scanner.get_line()))
    }

    fn match_single_quote(&mut self) -> Result<Token, LexerError> {
        let (line, column) = self.token_start_location();

        let Some(byte) = self.scanner.get_next()? else {
            return Err(LexerError::InvalidCharacterLiteral { line, column });
        };

        if byte == b'\'' || byte == b'\n' {
            return Err(LexerError::InvalidCharacterLiteral { line, column });
        }

        let character = byte;

        match self.scanner.get_next()? {
            Some(b'\'') => Ok(Token::new(TokenType::CharConst(character), line)),
            Some(_) | None => Err(LexerError::InvalidCharacterLiteral { line, column }),
        }
    }

    fn discard_next(&mut self) -> Result<(), LexerError> {
        let _ = self.scanner.get_next()?;
        Ok(())
    }

    fn single_char_token(&self, token_type: TokenType) -> Token {
        Token::new(token_type, self.scanner.get_line())
    }

    fn match_optional_equal(
        &mut self,
        single_type: TokenType,
        equal_type: TokenType,
    ) -> Result<Token, LexerError> {
        if self.scanner.peek_next() != Some(b'=') {
            return Ok(Token::new(single_type, self.scanner.get_line()));
        }

        self.discard_next()?;
        Ok(Token::new(equal_type, self.scanner.get_line()))
    }

    fn match_equal(&mut self) -> Result<Token, LexerError> {
        self.match_optional_equal(TokenType::Assign, TokenType::Eq)
    }

    fn match_greater(&mut self) -> Result<Token, LexerError> {
        self.match_optional_equal(TokenType::Gt, TokenType::Geq)
    }

    fn match_lesser(&mut self) -> Result<Token, LexerError> {
        self.match_optional_equal(TokenType::Lt, TokenType::Leq)
    }

    fn match_not(&mut self) -> Result<Token, LexerError> {
        self.match_optional_equal(TokenType::Not, TokenType::Neq)
    }

    fn match_and(&mut self) -> Result<Token, LexerError> {
        self.match_required_pair(b'&', TokenType::And)
    }

    fn match_or(&mut self) -> Result<Token, LexerError> {
        self.match_required_pair(b'|', TokenType::Or)
    }

    fn match_required_pair(
        &mut self,
        expected: u8,
        token_type: TokenType,
    ) -> Result<Token, LexerError> {
        let (line, column) = self.token_start_location();

        if self.scanner.peek_next() != Some(expected) {
            return Err(LexerError::InvalidLogicalOperator {
                character: expected,
                line,
                column,
            });
        }

        self.discard_next()?;

        Ok(Token::new(token_type, self.scanner.get_line()))
    }

    fn match_quotes(&mut self) -> Result<Token, LexerError> {
        let (line, column) = self.token_start_location();
        let mut buffer = Vec::new();

        loop {
            match self.scanner.get_next()? {
                Some(b'"') => {
                    return Ok(Token::new(TokenType::StringConst(buffer), line));
                }
                Some(b'\n') | None => {
                    return Err(LexerError::UnterminatedString { line, column });
                }
                Some(c) => buffer.push(c),
            }
        }
    }

    fn match_numeric(&mut self, initial: u8) -> Result<Token, LexerError> {
        let mut buffer = String::from(initial as char);
        let (line, column) = self.token_start_location();

        while let Some(next) = self.scanner.peek_next() {
            if !Self::is_allowed_numeric_character(next) {
                break;
            }

            self.discard_next()?;
            buffer.push(next as char);
        }

        let integer = buffer
            .parse::<i64>()
            .map_err(|_| LexerError::IntegerOutOfRange {
                lexeme: buffer.clone(),
                line,
                column,
            })?;

        Ok(Token::new(TokenType::IntegerConst(integer), line))
    }
}

impl<S: TScanner> TLexer for Lexer<S> {
    fn get_prox_token(&mut self) -> Result<Token, LexerError> {
        match self.get_next_meaningful_char()? {
            Some(initial) => self.lex_token(initial),
            None => Ok(Token::new(TokenType::Eof, self.scanner.get_line())),
        }
    }
}

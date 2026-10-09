use std::{error::Error, fmt};

use crate::{
    lexer::LexerError,
    parser::ParserError,
    scanner::ScannerError,
    semantic::SemanticError,
};

/// An error reported by a compiler frontend phase.
///
/// Preserves the original diagnostic for callers that need structured details.
#[derive(Debug)]
pub enum FrontendError {
    Scanner(ScannerError),
    Lexer(LexerError),
    Parser(ParserError),
    Semantic(SemanticError),
}

impl fmt::Display for FrontendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scanner(error) => write!(f, "{error}"),
            Self::Lexer(error) => write!(f, "{error}"),
            Self::Parser(error) => write!(f, "{error}"),
            Self::Semantic(error) => write!(f, "{error}"),
        }
    }
}

impl Error for FrontendError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Scanner(error) => Some(error),
            Self::Lexer(error) => Some(error),
            Self::Parser(error) => Some(error),
            Self::Semantic(error) => Some(error),
        }
    }
}

impl From<ScannerError> for FrontendError {
    fn from(error: ScannerError) -> Self {
        Self::Scanner(error)
    }
}

impl From<LexerError> for FrontendError {
    fn from(error: LexerError) -> Self {
        Self::Lexer(error)
    }
}

impl From<ParserError> for FrontendError {
    fn from(error: ParserError) -> Self {
        Self::Parser(error)
    }
}

impl From<SemanticError> for FrontendError {
    fn from(error: SemanticError) -> Self {
        Self::Semantic(error)
    }
}

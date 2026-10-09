use core::fmt;
use std::{
    error::Error,
    io::{self, BufReader, Read},
};

#[derive(Debug)]
pub enum ScannerError {
    Io(io::Error),
}

impl fmt::Display for ScannerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "scanner I/O error: {error}"),
        }
    }
}

impl Error for ScannerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
        }
    }
}

impl From<io::Error> for ScannerError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub trait TScanner {
    fn peek_next(&self) -> Option<u8>;
    fn get_next(&mut self) -> Result<Option<u8>, ScannerError>;
    fn get_line(&self) -> usize;
    fn get_column(&self) -> usize;
}

pub struct Scanner<R: Read> {
    reader: BufReader<R>,
    line: usize,
    column: usize,
    look_ahead: Option<u8>,
}

impl<R: Read> Scanner<R> {
    pub fn new(reader: R) -> io::Result<Self> {
        let mut reader = BufReader::new(reader);
        let look_ahead = Self::read_next(&mut reader)?;
        Ok(Self {
            reader,
            line: 1,
            column: 1,
            look_ahead,
        })
    }

    fn read_next(reader: &mut BufReader<R>) -> io::Result<Option<u8>> {
        let mut buffer = [0u8; 1];

        match reader.read(&mut buffer)? {
            0 => Ok(None),
            _ => Ok(Some(buffer[0])),
        }
    }
}

impl<R> TScanner for Scanner<R>
where
    R: Read,
{
    fn peek_next(&self) -> Option<u8> {
        self.look_ahead
    }

    fn get_next(&mut self) -> Result<Option<u8>, ScannerError> {
        let current = self.look_ahead;

        if let Some(byte) = current {
            if byte == b'\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.look_ahead = Self::read_next(&mut self.reader)?;
        }

        Ok(current)
    }

    fn get_line(&self) -> usize {
        self.line
    }

    fn get_column(&self) -> usize {
        self.column
    }
}

#[cfg(test)]
#[path = "tests/scanner.rs"]
mod tests;

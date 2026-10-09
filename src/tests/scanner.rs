use super::*;
use std::io::Cursor;

mod scanner_contract {
    use super::TScanner;

    pub fn empty_input_returns_none<S>(mut scanner: S)
    where
        S: TScanner,
    {
        assert_eq!(scanner.peek_next(), None);
        assert_eq!(scanner.get_next().unwrap(), None);
    }

    pub fn peek_does_not_consume_byte<S>(scanner: S)
    where
        S: TScanner,
    {
        assert_eq!(scanner.peek_next(), Some(b'a'));
        assert_eq!(scanner.peek_next(), Some(b'a'));
        assert_eq!(scanner.peek_next(), Some(b'a'));
    }

    pub fn get_next_consumes_current_byte<S>(mut scanner: S)
    where
        S: TScanner,
    {
        assert_eq!(scanner.get_next().unwrap(), Some(b'a'));
        assert_eq!(scanner.peek_next(), Some(b'b'));

        assert_eq!(scanner.get_next().unwrap(), Some(b'b'));
        assert_eq!(scanner.peek_next(), Some(b'c'));

        assert_eq!(scanner.get_next().unwrap(), Some(b'c'));
        assert_eq!(scanner.peek_next(), None);
    }

    pub fn get_next_returns_none_after_eof<S>(mut scanner: S)
    where
        S: TScanner,
    {
        assert_eq!(scanner.get_next().unwrap(), Some(b'a'));
        assert_eq!(scanner.get_next().unwrap(), None);
        assert_eq!(scanner.get_next().unwrap(), None);
    }

    pub fn peek_after_consuming_byte_returns_next_byte<S>(mut scanner: S)
    where
        S: TScanner,
    {
        assert_eq!(scanner.peek_next(), Some(b'a'));

        scanner.get_next().unwrap();

        assert_eq!(scanner.peek_next(), Some(b'b'));
    }

    pub fn line_starts_at_one<S>(scanner: S)
    where
        S: TScanner,
    {
        assert_eq!(scanner.get_line(), 1);
    }

    pub fn consuming_newline_increments_line<S>(mut scanner: S)
    where
        S: TScanner,
    {
        assert_eq!(scanner.get_line(), 1);

        assert_eq!(scanner.get_next().unwrap(), Some(b'a'));
        assert_eq!(scanner.get_line(), 1);

        assert_eq!(scanner.get_next().unwrap(), Some(b'\n'));
        assert_eq!(scanner.get_line(), 2);

        assert_eq!(scanner.peek_next(), Some(b'b'));
    }

    pub fn multiple_newlines_increment_line_correctly<S>(mut scanner: S)
    where
        S: TScanner,
    {
        assert_eq!(scanner.get_line(), 1);

        scanner.get_next().unwrap();
        assert_eq!(scanner.get_line(), 2);

        scanner.get_next().unwrap();
        assert_eq!(scanner.get_line(), 3);

        scanner.get_next().unwrap();
        assert_eq!(scanner.get_line(), 4);
    }

    pub fn peek_does_not_increment_line_on_newline<S>(scanner: S)
    where
        S: TScanner,
    {
        assert_eq!(scanner.peek_next(), Some(b'\n'));
        assert_eq!(scanner.get_line(), 1);

        assert_eq!(scanner.peek_next(), Some(b'\n'));
        assert_eq!(scanner.get_line(), 1);
    }

    pub fn scanner_preserves_byte_sequence<S>(mut scanner: S, expected: &[u8])
    where
        S: TScanner,
    {
        let mut output = Vec::new();

        while let Some(byte) = scanner.get_next().unwrap() {
            output.push(byte);
        }

        assert_eq!(output, expected);
    }

    pub fn get_line_after_scanning_line_break<S>(mut scanner: S)
    where
        S: TScanner,
    {
        let mut c = None;
        while c != Some(b'\n') {
            c = scanner.get_next().unwrap();
        }

        assert_eq!(scanner.get_line(), 2);
    }

    pub fn get_line_before_scanning_line_break<S>(mut scanner: S)
    where
        S: TScanner,
    {
        let mut c = None;
        while c != Some(b'o') {
            c = scanner.get_next().unwrap();
        }

        assert_eq!(scanner.get_line(), 1);
    }

    pub fn column_starts_at_one<S>(scanner: S)
    where
        S: TScanner,
    {
        assert_eq!(scanner.get_column(), 1);
    }

    pub fn consuming_byte_increments_column<S>(mut scanner: S)
    where
        S: TScanner,
    {
        assert_eq!(scanner.get_column(), 1);

        assert_eq!(scanner.get_next().unwrap(), Some(b'a'));
        assert_eq!(scanner.get_column(), 2);

        assert_eq!(scanner.get_next().unwrap(), Some(b'b'));
        assert_eq!(scanner.get_column(), 3);
    }

    pub fn peek_does_not_increment_column<S>(scanner: S)
    where
        S: TScanner,
    {
        assert_eq!(scanner.get_column(), 1);

        assert_eq!(scanner.peek_next(), Some(b'a'));
        assert_eq!(scanner.peek_next(), Some(b'a'));

        assert_eq!(scanner.get_column(), 1);
    }

    pub fn newline_resets_column<S>(mut scanner: S)
    where
        S: TScanner,
    {
        assert_eq!(scanner.get_next().unwrap(), Some(b'a'));
        assert_eq!(scanner.get_column(), 2);

        assert_eq!(scanner.get_next().unwrap(), Some(b'\n'));

        assert_eq!(scanner.get_line(), 2);
        assert_eq!(scanner.get_column(), 1);
    }

    pub fn columns_restart_after_newline<S>(mut scanner: S)
    where
        S: TScanner,
    {
        assert_eq!(scanner.get_next().unwrap(), Some(b'a'));
        assert_eq!(scanner.get_next().unwrap(), Some(b'\n'));

        assert_eq!(scanner.get_line(), 2);
        assert_eq!(scanner.get_column(), 1);

        assert_eq!(scanner.get_next().unwrap(), Some(b'b'));
        assert_eq!(scanner.get_column(), 2);
    }
}

fn scanner(input: &'static [u8]) -> Scanner<Cursor<&'static [u8]>> {
    Scanner::new(Cursor::new(input)).unwrap()
}

#[test]
fn scanner_empty_input_returns_none() {
    scanner_contract::empty_input_returns_none(scanner(b""));
}

#[test]
fn scanner_peek_does_not_consume_byte() {
    scanner_contract::peek_does_not_consume_byte(scanner(b"abc"));
}

#[test]
fn scanner_get_next_consumes_current_byte() {
    scanner_contract::get_next_consumes_current_byte(scanner(b"abc"));
}

#[test]
fn scanner_get_next_returns_none_after_eof() {
    scanner_contract::get_next_returns_none_after_eof(scanner(b"a"));
}

#[test]
fn scanner_peek_after_consuming_byte_returns_next_byte() {
    scanner_contract::peek_after_consuming_byte_returns_next_byte(scanner(b"ab"));
}

#[test]
fn scanner_line_starts_at_one() {
    scanner_contract::line_starts_at_one(scanner(b"abc"));
}

#[test]
fn scanner_consuming_newline_increments_line() {
    scanner_contract::consuming_newline_increments_line(scanner(b"a\nb"));
}

#[test]
fn scanner_multiple_newlines_increment_line_correctly() {
    scanner_contract::multiple_newlines_increment_line_correctly(scanner(b"\n\n\n"));
}

#[test]
fn scanner_peek_does_not_increment_line_on_newline() {
    scanner_contract::peek_does_not_increment_line_on_newline(scanner(b"\n"));
}

#[test]
fn scanner_preserves_byte_sequence() {
    let input = b"hello\nmicro c";
    scanner_contract::scanner_preserves_byte_sequence(scanner(input), input);
}

#[test]
fn scanner_get_line_after_scanning_line_break() {
    scanner_contract::get_line_after_scanning_line_break(scanner(b"hello\nmicro c"));
}

#[test]
fn scanner_get_line_before_scanning_line_break() {
    scanner_contract::get_line_before_scanning_line_break(scanner(b"hello\nmicro c"));
}

#[test]
fn scanner_column_starts_at_one() {
    scanner_contract::column_starts_at_one(scanner(b"not important"));
}

#[test]
fn scanner_consuming_byte_increments_column() {
    scanner_contract::consuming_byte_increments_column(scanner(b"ab"));
}

#[test]
fn scanner_peek_does_not_increment_column() {
    scanner_contract::peek_does_not_increment_column(scanner(b"a"));
}

#[test]
fn scanner_newline_resets_column() {
    scanner_contract::newline_resets_column(scanner(b"a\na"));
}

#[test]
fn scanner_columns_restart_after_newline() {
    scanner_contract::columns_restart_after_newline(scanner(b"a\nb"));
}

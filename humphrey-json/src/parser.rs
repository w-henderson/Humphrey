//! Provides the core JSON-parsing functionality.

use crate::error::{ParseError, TracebackError};
use crate::Value;

use std::borrow::Borrow;
use std::iter::Peekable;
use std::str::Chars;

const MAX_DEPTH: usize = 256;

impl Value {
    /// Parse a string into a JSON value.
    ///
    /// If unsuccessful, returns a `TracebackError`, giving information about the location of the syntax error within the JSON string.
    ///
    /// ## Usage
    /// ```
    /// let value = Value::parse("[1, 2, 3]");
    /// ```
    pub fn parse(s: impl AsRef<str>) -> Result<Self, TracebackError> {
        let chars = s.as_ref().chars();
        let mut parser = Parser::new(chars, MAX_DEPTH);
        let value = parser.parse_value()?;
        parser.expect_eof()?;

        Ok(value)
    }

    /// Parse a string into a JSON value with the specified maximum recursion depth.
    ///
    /// If unsuccessful, returns a `TracebackError`, giving information about the location of the syntax error within the JSON string.
    ///
    /// ## Usage
    /// ```
    /// let value = Value::parse_max_depth("[1, 2, 3]", 8);
    /// ```
    pub fn parse_max_depth(s: impl AsRef<str>, max_depth: usize) -> Result<Self, TracebackError> {
        let chars = s.as_ref().chars();
        let mut parser = Parser::new(chars, max_depth);
        let value = parser.parse_value()?;
        parser.expect_eof()?;

        Ok(value)
    }
}

/// Encapsulates the internal state of the parsing process.
struct Parser<'a> {
    chars: Peekable<Chars<'a>>,
    depth: usize,
    max_depth: usize,
    line: usize,
    column: usize,
    next_line: usize,
    next_column: usize,
}

impl<'a> Parser<'a> {
    /// Initialise a new parser.
    fn new(chars: Chars<'a>, max_depth: usize) -> Self {
        Self {
            chars: chars.peekable(),
            depth: 0,
            max_depth,
            line: 1,
            column: 1,
            next_line: 1,
            next_column: 1,
        }
    }

    /// Get the next character to be parsed.
    fn next(&mut self) -> Result<char, TracebackError> {
        if let Some(c) = self.chars.next() {
            self.line = self.next_line;
            self.column = self.next_column;

            if c == '\n' {
                self.next_line += 1;
                self.next_column = 0;
            } else if c != '\r' {
                self.next_column += 1;
            }

            return Ok(c);
        }

        Err(self.traceback(ParseError::UnexpectedEOF))
    }

    /// Convert a regular parsing error into a traceback error containing the location of the error.
    fn traceback(&self, e: ParseError) -> TracebackError {
        TracebackError {
            line: self.line,
            column: self.column,
            kind: e,
        }
    }

    /// Attempt to parse a value from the character stream.
    fn parse_value(&mut self) -> Result<Value, TracebackError> {
        self.flush_whitespace();

        match self.next() {
            Ok('"') => self.parse_string(),
            Ok('[') => self.parse_array(),
            Ok('{') => self.parse_object(),
            Ok(c) => self.parse_literal(c),
            Err(e) => Err(e),
        }
    }

    /// Attempt to parse a string from the character stream.
    fn parse_string(&mut self) -> Result<Value, TracebackError> {
        let mut string = String::with_capacity(256);
        let mut backslash = false;

        loop {
            let c = self.next()?;

            if backslash {
                match c {
                    '"' => string.push(0x22 as char),
                    '\\' => string.push(0x5c as char),
                    '/' => string.push(0x2f as char),
                    'b' => string.push(0x08 as char),
                    'f' => string.push(0x0c as char),
                    'n' => string.push(0x0a as char),
                    'r' => string.push(0x0d as char),
                    't' => string.push(0x09 as char),
                    'u' => {
                        let hex: String = [self.next()?, self.next()?, self.next()?, self.next()?]
                            .iter()
                            .collect();
                        let code = u16::from_str_radix(&hex, 16)
                            .map_err(|_| self.traceback(ParseError::InvalidEscapeSequence))?;

                        let new_char = if let Some(new_char) = char::from_u32(code as u32) {
                            new_char
                        } else {
                            quiet_assert(
                                self.next()? == '\\' && self.next()? == 'u',
                                self.traceback(ParseError::InvalidEscapeSequence),
                            )?;

                            let hex: String =
                                [self.next()?, self.next()?, self.next()?, self.next()?]
                                    .iter()
                                    .collect();
                            let code_2 = u16::from_str_radix(&hex, 16)
                                .map_err(|_| self.traceback(ParseError::InvalidEscapeSequence))?;

                            char::decode_utf16([code, code_2])
                                .next()
                                .ok_or_else(|| self.traceback(ParseError::InvalidEscapeSequence))?
                                .map_err(|_| self.traceback(ParseError::InvalidEscapeSequence))?
                        };

                        string.push(new_char);
                    }
                    _ => return Err(self.traceback(ParseError::InvalidEscapeSequence)),
                }

                backslash = false;
            } else if c == '\\' {
                backslash = true;
            } else if c == '"' {
                break;
            } else {
                match c as u32 {
                    0x20..=0x21 | 0x23..=0x5b | 0x5d..=0x10ffff => string.push(c),
                    _ => return Err(self.traceback(ParseError::InvalidToken)),
                }
            }
        }

        Ok(Value::String(string))
    }

    /// Attempt to parse an array from the character stream.
    fn parse_array(&mut self) -> Result<Value, TracebackError> {
        self.inc_depth()?;

        let mut array: Vec<Value> = Vec::with_capacity(16);

        loop {
            self.flush_whitespace();

            match self.chars.peek() {
                Some(&']') => {
                    if array.is_empty() {
                        break;
                    } else {
                        return Err(self.traceback(ParseError::TrailingComma));
                    }
                }
                Some(_) => array.push(self.parse_value()?),
                None => return Err(self.traceback(ParseError::UnexpectedEOF)),
            }

            self.flush_whitespace();

            match self.chars.peek() {
                Some(&',') => (),
                Some(&']') => break,
                Some(_) => return Err(self.traceback(ParseError::InvalidToken)),
                None => return Err(self.traceback(ParseError::UnexpectedEOF)),
            }

            self.next()?;
        }

        self.next()?;
        self.dec_depth();

        Ok(Value::Array(array))
    }

    /// Attempt to parse an object from the character stream.
    fn parse_object(&mut self) -> Result<Value, TracebackError> {
        self.inc_depth()?;

        let mut object: Vec<(String, Value)> = Vec::with_capacity(16);
        let mut trailing_comma = false;

        loop {
            self.flush_whitespace();

            match self.chars.peek() {
                Some(&'}') => {
                    if trailing_comma {
                        return Err(self.traceback(ParseError::TrailingComma));
                    } else {
                        break;
                    }
                }
                Some(&',') => {
                    if trailing_comma {
                        return Err(self.traceback(ParseError::InvalidToken));
                    } else {
                        trailing_comma = true;

                        if object.is_empty() {
                            return Err(self.traceback(ParseError::InvalidToken));
                        }

                        self.next()?;
                    }
                }
                Some(_) => {
                    trailing_comma = false;
                    let string_start = self.next()?;
                    quiet_assert(
                        string_start == '"',
                        self.traceback(ParseError::InvalidToken),
                    )?;

                    let key = self.parse_string()?.as_str().unwrap().to_string();
                    self.flush_whitespace();

                    let sep = self.next()?;
                    quiet_assert(sep == ':', self.traceback(ParseError::InvalidToken))?;
                    self.flush_whitespace();

                    let value = self.parse_value()?;

                    object.push((key, value));
                }
                None => return Err(self.traceback(ParseError::UnexpectedEOF)),
            }
        }

        self.next()?;
        self.dec_depth();

        Ok(Value::Object(object))
    }

    /// Attempt to parse a literal from the character stream.
    fn parse_literal(&mut self, c: char) -> Result<Value, TracebackError> {
        let mut string = String::from(c);

        while self.chars.peek().map_or(false, |&c| is_literal(c)) {
            string.push(self.next().unwrap());
        }

        match string.as_str() {
            "null" => Ok(Value::Null),
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            number => Ok(Value::Number(
                number
                    .parse()
                    .map_err(|_| self.traceback(ParseError::InvalidToken))?,
            )),
        }
    }

    /// Assert that there are no more characters to be parsed, or return an error.
    fn expect_eof(&mut self) -> Result<(), TracebackError> {
        self.flush_whitespace();

        match self.chars.peek() {
            Some(_) => Err(self.traceback(ParseError::InvalidToken)),
            None => Ok(()),
        }
    }

    /// Fast-forward the iterator until the next character is not whitespace.
    fn flush_whitespace(&mut self) {
        while self.chars.peek().map_or(false, is_whitespace) {
            self.next().ok();
        }
    }

    fn inc_depth(&mut self) -> Result<(), TracebackError> {
        if self.depth == self.max_depth {
            Err(self.traceback(ParseError::RecursionDepthExceeded))
        } else {
            self.depth += 1;
            Ok(())
        }
    }

    fn dec_depth(&mut self) {
        self.depth -= 1;
    }
}

/// Assert a condition, or return an error.
fn quiet_assert(condition: bool, error: TracebackError) -> Result<(), TracebackError> {
    if condition {
        Ok(())
    } else {
        Err(error)
    }
}

/// Check whether a character is whitespace according to the specification.
fn is_whitespace(c: impl Borrow<char>) -> bool {
    matches!(c.borrow(), ' ' | '\t' | '\n' | '\r')
}

/// Check whether the character is reserved.
fn is_literal(c: impl Borrow<char>) -> bool {
    let c = c.borrow();
    !is_whitespace(c) && *c != ',' && *c != '}' && *c != ']'
}

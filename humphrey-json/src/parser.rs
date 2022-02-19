use crate::error::{ParseError, TracebackError};
use crate::Value;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

impl Value {
    pub fn parse(s: impl AsRef<str>) -> Result<Self, TracebackError> {
        let chars = s.as_ref().chars();
        let mut parser = Parser::new(chars);
        parser.parse_value()
    }
}

struct Parser<'a> {
    chars: Peekable<Chars<'a>>,
    line: usize,
    column: usize,
}

impl<'a> Parser<'a> {
    fn new(chars: Chars<'a>) -> Self {
        Self {
            chars: chars.peekable(),
            line: 1,
            column: 1,
        }
    }

    fn next(&mut self) -> Result<char, TracebackError> {
        if let Some(c) = self.chars.next() {
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }

            return Ok(c);
        }

        Err(self.traceback(ParseError::UnexpectedEOF))
    }

    fn traceback(&self, e: ParseError) -> TracebackError {
        TracebackError {
            line: self.line,
            column: self.column,
            kind: e,
        }
    }

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

    fn parse_string(&mut self) -> Result<Value, TracebackError> {
        let mut string = String::new();
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
                        let code = u32::from_str_radix(&hex, 16)
                            .map_err(|_| self.traceback(ParseError::InvalidEscapeSequence))?;
                        let new_char = char::from_u32(code)
                            .ok_or_else(|| self.traceback(ParseError::InvalidEscapeSequence))?;

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
                string.push(c);
            }
        }

        Ok(Value::String(string))
    }

    fn parse_array(&mut self) -> Result<Value, TracebackError> {
        let mut array: Vec<Value> = Vec::new();

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

        Ok(Value::Array(array))
    }

    fn parse_object(&mut self) -> Result<Value, TracebackError> {
        let mut object = HashMap::new();
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
                    trailing_comma = true;

                    if object.is_empty() {
                        return Err(self.traceback(ParseError::InvalidToken));
                    }
                }
                Some(_) => {
                    trailing_comma = false;
                    self.next()?;

                    let key = self.parse_string()?.as_str().unwrap().to_string();
                    self.flush_whitespace();

                    let sep = self.next()?;
                    quiet_assert(sep == ':', self.traceback(ParseError::InvalidToken))?;
                    self.flush_whitespace();

                    let value = self.parse_value()?;

                    object.insert(key, value);
                }
                None => return Err(self.traceback(ParseError::UnexpectedEOF)),
            }

            self.next()?;
        }

        self.next()?;

        Ok(Value::Object(object))
    }

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

    fn flush_whitespace(&mut self) {
        while self.chars.peek().map_or(false, is_whitespace) {
            self.next().ok();
        }
    }
}

fn quiet_assert(condition: bool, error: TracebackError) -> Result<(), TracebackError> {
    if condition {
        Ok(())
    } else {
        Err(error)
    }
}

fn is_whitespace(c: impl Borrow<char>) -> bool {
    matches!(c.borrow(), ' ' | '\t' | '\n' | '\r')
}

fn is_literal(c: impl Borrow<char>) -> bool {
    let c = c.borrow();
    !is_whitespace(c) && *c != ',' && *c != '}' && *c != ']'
}

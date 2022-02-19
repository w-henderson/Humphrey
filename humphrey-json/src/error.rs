use std::error::Error;
use std::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseError {
    UnknownError,
    InvalidToken,
    UnexpectedEOF,
    InvalidEscapeSequence,
    TrailingComma,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TracebackError {
    pub(crate) line: usize,
    pub(crate) column: usize,
    pub(crate) kind: ParseError,
}

impl Display for TracebackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error at {}:{}: {:?}", self.line, self.column, self.kind)
    }
}

impl Error for TracebackError {}

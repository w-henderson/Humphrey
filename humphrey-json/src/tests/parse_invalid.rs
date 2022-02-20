use crate::error::{ParseError, TracebackError};
use crate::Value;

#[test]
fn test_trailing_comma() {
    let string = include_str!("./testcases/trailing_comma.json");
    let error = Value::parse(string).unwrap_err();

    let expected_error = TracebackError {
        line: 2,
        column: 20,
        kind: ParseError::TrailingComma,
    };

    assert_eq!(error, expected_error);
}

#[test]
fn test_invalid_token() {
    let string = "{!}";
    let error = Value::parse(string).unwrap_err();

    let expected_error = TracebackError {
        line: 1,
        column: 2,
        kind: ParseError::InvalidToken,
    };

    assert_eq!(error, expected_error);
}

#[test]
fn test_unexpected_eof() {
    let string = "{\"key\": \"value\"";
    let error = Value::parse(string).unwrap_err();

    let expected_error = TracebackError {
        line: 1,
        column: string.len(),
        kind: ParseError::UnexpectedEOF,
    };

    assert_eq!(error, expected_error);
}

#[test]
fn test_invalid_escape_sequence() {
    let string = "\"\\!\"";
    let error = Value::parse(string).unwrap_err();

    let expected_error = TracebackError {
        line: 1,
        column: 3,
        kind: ParseError::InvalidEscapeSequence,
    };

    assert_eq!(error, expected_error);
}

#[test]
fn test_trailing_garbage() {
    let string = "\"value\" garbage";
    let error = Value::parse(string).unwrap_err();

    let expected_error = TracebackError {
        line: 1,
        column: 8,
        kind: ParseError::InvalidToken,
    };

    assert_eq!(error, expected_error);
}

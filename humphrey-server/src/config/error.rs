use std::error::Error;
use std::fmt::Display;

/// Represents an error encountered during configuration parsing.
#[derive(Debug, PartialEq, Eq)]
pub struct ConfigError {
    message: &'static str,
    line: i64,
}

impl ConfigError {
    /// Creates a new configuration error object.
    pub fn new(message: &'static str, line: i64) -> Self {
        Self { message, line }
    }
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Configuration error on line {}: {}",
            self.line, self.message
        )
    }
}

impl Error for ConfigError {}

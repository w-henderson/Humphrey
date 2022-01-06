//! Provides error handling functionality for the configuration parser.

use std::error::Error;
use std::fmt::Display;

/// Represents an error encountered during configuration parsing.
#[derive(Debug, PartialEq, Eq)]
pub struct ConfigError {
    message: &'static str,
    file: String,
    line: u64,
}

impl ConfigError {
    /// Creates a new configuration error object.
    pub fn new(message: &'static str, file: &str, line: u64) -> Self {
        Self {
            message,
            file: file.to_string(),
            line,
        }
    }
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Configuration error at {} line {}: {}",
            self.file, self.line, self.message
        )
    }
}

impl Error for ConfigError {}

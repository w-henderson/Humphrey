//! Provides error handling for the WebSocket crate.

use std::error::Error;
use std::fmt::Display;

/// Represents a WebSocket error.
#[derive(Debug, PartialEq, Eq)]
pub enum WebsocketError {
    /// An error occurred when reading from the stream.
    ReadError,
    /// An error occurred when writing to the stream.
    WriteError,
    /// An error occurred during the WebSocket handshake.
    HandshakeError,
    /// The frame opcode was invalid.
    InvalidOpcode,
    /// The connection has been closed so the request could not be completed.
    ConnectionClosed,
}

impl Display for WebsocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for WebsocketError {}

use std::error::Error;
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub enum WebsocketError {
    ReadError,
    WriteError,
    HandshakeError,
    InvalidOpcode,
    ConnectionClosed,
}

impl Display for WebsocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for WebsocketError {}

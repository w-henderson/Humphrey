//! Provides functionality for working with a WebSocket stream.

use humphrey::stream::Stream;

use crate::error::WebsocketError;
use crate::frame::{Frame, Opcode};
use crate::message::Message;
use crate::restion::Restion;

use std::io::{Read, Write};

/// Represents a WebSocket stream.
pub struct WebsocketStream<T>
where
    T: Read + Write,
{
    stream: T,
    closed: bool,
}

impl<T> WebsocketStream<T>
where
    T: Read + Write,
{
    /// Creates a new `WebsocketStream` wrapping an underlying stream, usually `TcpStream`.
    ///
    /// When the `WebsocketStream` is dropped, a close frame will be sent to the client.
    pub fn new(stream: T) -> Self {
        Self {
            stream,
            closed: false,
        }
    }

    /// Blocks until a message is received from the client.
    pub fn recv(&mut self) -> Result<Message, WebsocketError> {
        let message = Message::from_stream(&mut self.stream);

        if let Err(WebsocketError::ConnectionClosed) = message {
            self.closed = true;
        }

        message
    }

    /// Sends a message to the client.
    pub fn send(&mut self, message: Message) -> Result<(), WebsocketError> {
        self.stream
            .write_all(&message.to_bytes())
            .map_err(|_| WebsocketError::WriteError)
    }

    /// Returns a mutable reference to the underlying stream.
    pub fn inner(&mut self) -> &mut T {
        &mut self.stream
    }
}

impl WebsocketStream<Stream<'_>> {
    /// Attemps to receive a message from the stream without blocking.
    pub fn recv_nonblocking(&mut self) -> Restion<Message, WebsocketError> {
        let message = Message::from_stream_nonblocking(&mut self.stream);

        if let Restion::Err(WebsocketError::ConnectionClosed) = message {
            self.closed = true;
        }

        message
    }
}

impl<T> Drop for WebsocketStream<T>
where
    T: Read + Write,
{
    fn drop(&mut self) {
        if !self.closed {
            self.stream
                .write_all(Frame::new(Opcode::Close, Vec::new()).as_ref())
                .ok();
        }
    }
}

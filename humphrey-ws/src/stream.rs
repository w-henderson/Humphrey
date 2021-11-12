use crate::error::WebsocketError;
use crate::frame::{Frame, Opcode};
use crate::message::Message;

use std::io::{Read, Write};

/// Represents a WebSocket stream.
///
/// Implements `Iterator` over `Message`s for reading messages from the stream.
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

    /// Sends a message to the client.
    pub fn send(&mut self, message: Message) -> Result<(), WebsocketError> {
        self.stream
            .write_all(&message.to_bytes())
            .map_err(|_| WebsocketError::WriteError)
    }
}

impl<T> Iterator for WebsocketStream<T>
where
    T: Read + Write,
{
    type Item = Result<Message, WebsocketError>;

    fn next(&mut self) -> Option<Self::Item> {
        let message = Message::from_stream(&mut self.stream);

        if let Err(WebsocketError::ConnectionClosed) = message {
            self.closed = true;
            None
        } else {
            Some(message)
        }
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

            self.find(|message| matches!(message, Err(WebsocketError::ConnectionClosed)));
        }
    }
}

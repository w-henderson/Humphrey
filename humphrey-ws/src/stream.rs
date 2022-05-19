//! Provides functionality for working with a WebSocket stream.

use humphrey::stream::Stream;

use crate::error::WebsocketError;
use crate::frame::{Frame, Opcode};
use crate::message::Message;
use crate::restion::Restion;

use std::io::{Read, Write};
use std::net::SocketAddr;
use std::time::Instant;

/// Represents a WebSocket stream.
///
/// Messages can be sent and received through the `send` and `recv` methods.
///
/// The stream also implements the `Read` and `Write` traits to help with compatibility with
///   other crates. These simply wrap and unwrap the bytes in WebSocket frames.
pub struct WebsocketStream {
    pub(crate) stream: Stream,
    pub(crate) closed: bool,
    pub(crate) last_pong: Instant,
}

impl WebsocketStream {
    /// Creates a new `WebsocketStream` wrapping an underlying Humphrey stream.
    ///
    /// When the `WebsocketStream` is dropped, a close frame will be sent to the client.
    pub fn new(stream: Stream) -> Self {
        Self {
            stream,
            closed: false,
            last_pong: Instant::now(),
        }
    }

    /// Blocks until a message is received from the client.
    pub fn recv(&mut self) -> Result<Message, WebsocketError> {
        let message = Message::from_stream(self);

        if let Err(WebsocketError::ConnectionClosed) = message {
            self.closed = true;
        }

        message
    }

    /// Attempts to receive a message from the stream without blocking.
    pub fn recv_nonblocking(&mut self) -> Restion<Message, WebsocketError> {
        let message = Message::from_stream_nonblocking(self);

        if let Restion::Err(WebsocketError::ConnectionClosed) = message {
            self.closed = true;
        }

        message
    }

    /// Sends a message to the client.
    pub fn send(&mut self, message: Message) -> Result<(), WebsocketError> {
        self.send_raw(message.to_frame())
    }

    /// Sends a ping to the client.
    pub fn ping(&mut self) -> Result<(), WebsocketError> {
        let bytes: Vec<u8> = Frame::new(Opcode::Ping, Vec::new()).into();
        self.send_raw(bytes)
    }

    /// Sends a raw frame to the client.
    ///
    /// ## Warning
    /// This function does not check that the frame is valid.
    pub(crate) fn send_raw(&mut self, bytes: impl AsRef<[u8]>) -> Result<(), WebsocketError> {
        self.stream
            .write_all(bytes.as_ref())
            .map_err(|_| WebsocketError::WriteError)
    }

    /// Attempts to get the peer address of this stream.
    pub fn peer_addr(&self) -> Result<SocketAddr, std::io::Error> {
        self.stream.peer_addr()
    }

    /// Returns a mutable reference to the underlying stream.
    pub fn inner(&mut self) -> &mut Stream {
        &mut self.stream
    }
}

impl Read for WebsocketStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if let Ok(message) = self.recv() {
            let bytes = message.bytes();

            if bytes.len() <= buf.len() {
                buf[..bytes.len()].copy_from_slice(bytes);
                Ok(bytes.len())
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Buffer is too small",
                ))
            }
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to receive message",
            ))
        }
    }
}

impl Write for WebsocketStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let message = Message::new(buf);

        if self.send(message).is_ok() {
            Ok(buf.len())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to send message",
            ))
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Drop for WebsocketStream {
    fn drop(&mut self) {
        if !self.closed {
            self.stream
                .write_all(Frame::new(Opcode::Close, Vec::new()).as_ref())
                .ok();
        }
    }
}

use crate::error::WebsocketError;
use crate::frame::{Frame, Opcode};

use std::io::{Read, Write};

/// Represents a WebSocket message.
pub struct Message {
    payload: Vec<u8>,
    text: bool,
}

impl Message {
    /// Attemps to read a message from the given stream.
    ///
    /// Silently responds to pings with pongs, as specified in [RFC 6455 Section 5.5.2](https://datatracker.ietf.org/doc/html/rfc6455#section-5.5.2).
    pub fn from_stream<T>(mut stream: T) -> Result<Self, WebsocketError>
    where
        T: Read + Write,
    {
        let mut frames: Vec<Frame> = Vec::new();

        // Keep reading frames until we get the finish frame
        while frames.last().map(|f| f.fin).unwrap_or(false) {
            let frame = Frame::from_stream(&mut stream)?;

            // If this is a ping, respond with a pong
            if frame.opcode == Opcode::Ping {
                let pong = Frame::new(Opcode::Pong, frame.payload);
                stream
                    .write_all(pong.as_ref())
                    .map_err(|_| WebsocketError::WriteError)?;
                continue;
            }

            // If this closes the connection, return the error
            if frame.opcode == Opcode::Close {
                let close = Frame::new(Opcode::Close, frame.payload);
                stream
                    .write_all(close.as_ref())
                    .map_err(|_| WebsocketError::WriteError)?;
                return Err(WebsocketError::ConnectionClosed);
            }

            frames.push(frame);
        }

        // Concatenate the payloads of all frames into a single payload
        let payload = frames.iter().fold(Vec::new(), |mut acc, frame| {
            acc.extend(frame.payload.iter());
            acc
        });

        Ok(Self {
            payload,
            text: frames
                .first()
                .map(|f| f.opcode == Opcode::Text)
                .unwrap_or(false),
        })
    }

    /// Returns whether the sender of this message specified that it contains text.
    pub fn is_text(&self) -> bool {
        self.text
    }

    /// Returns the payload as a string, if possible.
    ///
    /// If the opcode is `Opcode::Text` (`0x1`), but the payload is not valid UTF-8, the function will return `None`.
    /// Otherwise, it will not attempt to convert the payload to a string and will immediately return `None`.
    pub fn text(&self) -> Option<&str> {
        if self.text {
            std::str::from_utf8(&self.payload).ok()
        } else {
            None
        }
    }

    /// Returns the payload as a slice of bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.payload
    }
}

impl AsRef<[u8]> for Message {
    fn as_ref(&self) -> &[u8] {
        &self.payload
    }
}

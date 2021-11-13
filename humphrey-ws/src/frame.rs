use crate::error::WebsocketError;

use std::convert::TryFrom;
use std::io::Read;

/// Represents a frame of WebSocket data.
/// Follows [Section 5.2 of RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455#section-5.2)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub(crate) fin: bool,
    pub(crate) rsv: [bool; 3],
    pub(crate) opcode: Opcode,
    pub(crate) mask: bool,
    pub(crate) length: u64,
    pub(crate) masking_key: [u8; 4],
    pub(crate) payload: Vec<u8>,
}

/// Represents the type of WebSocket frame.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    Continuation = 0x0,
    Text = 0x1,
    Binary = 0x2,
    Close = 0x8,
    Ping = 0x9,
    Pong = 0xA,
}

impl TryFrom<u8> for Opcode {
    type Error = WebsocketError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x0 => Ok(Self::Continuation),
            0x1 => Ok(Self::Text),
            0x2 => Ok(Self::Binary),
            0x8 => Ok(Self::Close),
            0x9 => Ok(Self::Ping),
            0xA => Ok(Self::Pong),
            _ => Err(WebsocketError::InvalidOpcode),
        }
    }
}

impl Frame {
    /// Creates a new frame with the given parameters.
    /// Does not mask the payload.
    pub fn new(opcode: Opcode, payload: Vec<u8>) -> Self {
        Self {
            fin: true,
            rsv: [false; 3],
            opcode,
            mask: false,
            length: payload.len() as u64,
            masking_key: [0; 4],
            payload,
        }
    }

    /// Attemps to read a frame from the given stream.
    pub fn from_stream<T>(mut stream: T) -> Result<Self, WebsocketError>
    where
        T: Read,
    {
        let mut buf: [u8; 2] = [0; 2];
        stream
            .read_exact(&mut buf)
            .map_err(|_| WebsocketError::ReadError)?;

        // Parse header information
        let fin = buf[0] & 0x80 != 0;
        let rsv = [buf[0] & 0x40 != 0, buf[0] & 0x20 != 0, buf[0] & 0x10 != 0];
        let opcode = Opcode::try_from(buf[0] & 0xF)?;
        let mask = buf[1] & 0x80 != 0;

        let mut length: u64 = (buf[1] & 0x7F) as u64;
        if length == 126 {
            stream
                .read_exact(&mut buf)
                .map_err(|_| WebsocketError::ReadError)?;
            length = u16::from_be_bytes(buf) as u64;
        } else if length == 127 {
            let mut buf: [u8; 8] = [0; 8];
            stream
                .read_exact(&mut buf)
                .map_err(|_| WebsocketError::ReadError)?;
            length = u64::from_be_bytes(buf);
        }

        let masking_key = {
            let mut buf: [u8; 4] = [0; 4];
            if mask {
                stream
                    .read_exact(&mut buf)
                    .map_err(|_| WebsocketError::ReadError)?;
            }
            buf
        };

        // Read the payload
        let mut payload: Vec<u8> = vec![0; length as usize];
        stream
            .read_exact(&mut payload)
            .map_err(|_| WebsocketError::ReadError)?;

        // Unmask the payload
        payload
            .iter_mut()
            .enumerate()
            .for_each(|(i, tem)| *tem ^= masking_key[i % 4]);

        Ok(Self {
            fin,
            rsv,
            opcode,
            mask,
            length,
            masking_key,
            payload,
        })
    }
}

impl From<Frame> for Vec<u8> {
    fn from(f: Frame) -> Self {
        let mut buf: Vec<u8> = vec![0; 2];

        // Set the header bits
        buf[0] = (f.fin as u8) << 7
            | (f.rsv[0] as u8) << 6
            | (f.rsv[1] as u8) << 5
            | (f.rsv[2] as u8) << 4
            | f.opcode as u8;

        // Set the length information
        if f.length < 126 {
            buf[1] = (f.mask as u8) << 7 | f.length as u8;
        } else if f.length < 65536 {
            buf[1] = (f.mask as u8) << 7 | 126;
            buf.extend_from_slice(&(f.length as u16).to_be_bytes());
        } else {
            buf[1] = (f.mask as u8) << 7 | 127;
            buf.extend_from_slice(&(f.length as u64).to_be_bytes());
        }

        // Add the masking key (if required)
        if f.mask {
            buf.extend_from_slice(&f.masking_key);
        }

        // Add the payload and return
        buf.extend_from_slice(&f.payload);
        buf
    }
}

impl AsRef<[u8]> for Frame {
    fn as_ref(&self) -> &[u8] {
        &self.payload
    }
}

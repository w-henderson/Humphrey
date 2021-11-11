use std::convert::TryFrom;
use std::error::Error;
use std::io::Read;

/// Represents a frame of WebSocket data.
/// Follows [Section 5.2 of RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455#section-5.2)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    fin: bool,
    rsv: [bool; 3],
    opcode: Opcode,
    mask: bool,
    length: u64,
    masking_key: [u8; 4],
    payload: Vec<u8>,
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
    type Error = Box<dyn Error>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x0 => Ok(Self::Continuation),
            0x1 => Ok(Self::Text),
            0x2 => Ok(Self::Binary),
            0x8 => Ok(Self::Close),
            0x9 => Ok(Self::Ping),
            0xA => Ok(Self::Pong),
            _ => Err("Invalid opcode".into()),
        }
    }
}

impl Frame {
    /// Attemps to read a frame from the given stream.
    pub fn from_stream<T>(mut stream: T) -> Result<Self, Box<dyn Error>>
    where
        T: Read,
    {
        let mut buf: [u8; 2] = [0; 2];
        stream.read_exact(&mut buf)?;

        // Parse header information
        let fin = buf[0] & 0x80 != 0;
        let rsv = [buf[0] & 0x40 != 0, buf[0] & 0x20 != 0, buf[0] & 0x10 != 0];
        let opcode = Opcode::try_from(buf[0] & 0xF)?;
        let mask = buf[1] & 0x80 != 0;

        let mut length: u64 = (buf[1] as u8 & 0x7F) as u64;
        if length == 126 {
            stream.read_exact(&mut buf)?;
            length = u16::from_be_bytes(buf) as u64;
        } else if length == 127 {
            let buf: [u8; 8] = [0; 8];
            length = u64::from_be_bytes(buf);
        }

        let masking_key = {
            let mut buf: [u8; 4] = [0; 4];
            if mask {
                stream.read_exact(&mut buf)?;
            }
            buf
        };

        // Read the payload
        let mut payload: Vec<u8> = vec![0; length as usize];
        stream.read_exact(&mut payload)?;

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

    /// Returns the payload as a string, if possible.
    ///
    /// If the payload opcode is `Opcode::Text` (`0x1`), but the payload is not valid UTF-8, the function will return `None`.
    /// Otherwise, it will not attempt to convert the payload to a string and will immediately return `None`.
    pub fn text(&self) -> Option<String> {
        if self.opcode != Opcode::Text {
            return None;
        }

        String::from_utf8(self.payload.clone()).ok()
    }
}

impl AsRef<[u8]> for Frame {
    fn as_ref(&self) -> &[u8] {
        &self.payload
    }
}

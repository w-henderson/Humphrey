use crate::fcgi::types::FcgiType;
use crate::fcgi::types::{FCGI_HEADER_SIZE, FCGI_VERSION};

use std::convert::TryInto;
use std::error::Error;
use std::io::Read;

/// Represents an FCGI record, a component of a transmission.
#[derive(Debug)]
pub struct FcgiRecord {
    pub version: u8,
    pub fcgi_type: FcgiType,
    pub request_id: u16,
    pub content_length: u16,
    pub padding_length: u8,
    pub reserved: u8,
    pub content_data: Vec<u8>,
    pub padding_data: Vec<u8>,
}

impl FcgiRecord {
    /// Create a new FCGI record with the given parameters and infer the rest.
    pub fn new(fcgi_type: FcgiType, content: &[u8], request_id: u16) -> Self {
        Self {
            version: FCGI_VERSION,
            fcgi_type,
            request_id,
            content_length: content.len() as u16,
            padding_length: 0,
            reserved: 0,
            content_data: content.to_vec(),
            padding_data: Vec::new(),
        }
    }

    /// Return a "begin" record
    pub fn begin_record(request_id: u16, keep_alive: bool) -> Self {
        FcgiRecord::new(
            FcgiType::Begin,
            &[
                0,
                1, // responder
                keep_alive as u8,
                0,
                0,
                0,
                0,
                0,
            ],
            request_id,
        )
    }

    /// Reads a record from a readable type.
    pub fn read_from<T>(mut stream: T) -> Result<Self, Box<dyn Error>>
    where
        T: Read,
    {
        let mut header: [u8; FCGI_HEADER_SIZE] = [0; FCGI_HEADER_SIZE];
        stream.read_exact(&mut header)?;

        let version = header[0];
        let fcgi_type: FcgiType = header[1].try_into().unwrap();
        let request_id = u16::from_be_bytes(header[2..4].try_into()?);
        let content_length = u16::from_be_bytes(header[4..6].try_into()?);
        let padding_length = header[6];
        let reserved = header[7];

        let mut content: Vec<u8> = vec![0; content_length as usize];
        stream.read_exact(&mut content)?;

        let mut padding: Vec<u8> = vec![0; padding_length as usize];
        stream.read_exact(&mut padding)?;

        Ok(Self {
            version,
            fcgi_type,
            request_id,
            content_length,
            padding_length,
            reserved,
            content_data: content,
            padding_data: padding,
        })
    }
}

impl Into<Vec<u8>> for FcgiRecord {
    fn into(self) -> Vec<u8> {
        let length = FCGI_HEADER_SIZE as u16 + self.content_length + self.padding_length as u16;
        let mut result: Vec<u8> = Vec::with_capacity(length as usize);

        result.push(self.version);
        result.push(self.fcgi_type as u8);
        result.extend(self.request_id.to_be_bytes());
        result.extend(self.content_length.to_be_bytes());
        result.push(self.padding_length);
        result.push(self.reserved);
        result.extend(self.content_data);
        result.extend(self.padding_data);

        result
    }
}

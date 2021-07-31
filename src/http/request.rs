use super::headers::{RequestHeader, RequestHeaderMap};
use super::method::Method;
use std::error::Error;
use std::io::{BufRead, BufReader, Read};

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub url: String,
    pub version: String,
    pub headers: RequestHeaderMap,
    pub content: Option<Vec<u8>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RequestError {
    Request,
    Stream,
}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", "RequestError")
    }
}

impl Error for RequestError {}

impl Request {
    pub fn from_stream(stream: &mut impl Read) -> Result<Self, RequestError> {
        let mut reader = BufReader::new(stream);
        let mut start_line_buf: Vec<u8> = Vec::new();
        reader
            .read_until(0xA, &mut start_line_buf)
            .map_err(|_| RequestError::Stream)?;

        let start_line_string =
            String::from_utf8(start_line_buf).map_err(|_| RequestError::Request)?;
        let start_line: Vec<&str> = start_line_string.split(" ").collect();

        safe_assert(start_line.len() == 3)?;

        let method = Method::from_name(start_line[0])?;
        let url = start_line[1].to_string();
        let version = start_line[2].to_string().replace("\r\n", "");

        let mut headers = RequestHeaderMap::new();

        loop {
            let mut line_buf: Vec<u8> = Vec::new();
            reader
                .read_until(0xA, &mut line_buf)
                .map_err(|_| RequestError::Stream)?;
            let line = String::from_utf8(line_buf).map_err(|_| RequestError::Request)?;

            if line == "\r\n" {
                break;
            } else {
                safe_assert(line.len() >= 2)?;
                let line_without_crlf = &line[0..line.len() - 2];
                let line_parts: Vec<&str> = line_without_crlf.splitn(2, ':').collect();
                headers.insert(
                    RequestHeader::from(line_parts[0]),
                    line_parts[1].trim_start().to_string(),
                );
            }
        }

        if let Some(content_length) = headers.get(&RequestHeader::ContentLength) {
            let content_length: usize =
                content_length.parse().map_err(|_| RequestError::Request)?;
            let mut content_buf: Vec<u8> = Vec::with_capacity(content_length);
            reader
                .read_exact(&mut content_buf)
                .map_err(|_| RequestError::Stream)?;

            Ok(Self {
                method,
                url,
                version,
                headers,
                content: Some(content_buf),
            })
        } else {
            Ok(Self {
                method,
                url,
                version,
                headers,
                content: None,
            })
        }
    }
}

fn safe_assert(condition: bool) -> Result<(), RequestError> {
    match condition {
        true => Ok(()),
        false => Err(RequestError::Request),
    }
}

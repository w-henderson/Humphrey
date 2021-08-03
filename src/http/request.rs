use crate::http::headers::{RequestHeader, RequestHeaderMap};
use crate::http::method::Method;
use std::error::Error;
use std::io::{BufRead, BufReader, Read};

/// Represents a request to the server.
/// Contains parsed information about the request's data.
#[derive(Debug)]
pub struct Request {
    /// The method used in making the request, e.g. "GET".
    pub method: Method,
    /// The URI to which the request was made.
    pub uri: String,
    /// The HTTP version of the request.
    pub version: String,
    /// A map of headers included in the request.
    pub headers: RequestHeaderMap,
    /// The request body, if supplied.
    pub content: Option<Vec<u8>>,
}

/// An error which occurred during the parsing of a request.
#[derive(Debug, PartialEq, Eq)]
pub enum RequestError {
    /// The request could not be parsed due to invalid data.
    Request,
    /// The request could not be parsed due to an issue with the stream.
    Stream,
}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", "RequestError")
    }
}

impl Error for RequestError {}

impl Request {
    /// Attempts to read and parse one HTTP request from the given stream.
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
        let uri = start_line[1].to_string();
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
                uri,
                version,
                headers,
                content: Some(content_buf),
            })
        } else {
            Ok(Self {
                method,
                uri,
                version,
                headers,
                content: None,
            })
        }
    }
}

/// Asserts that the condition is true, returning a `Result`.
fn safe_assert(condition: bool) -> Result<(), RequestError> {
    match condition {
        true => Ok(()),
        false => Err(RequestError::Request),
    }
}

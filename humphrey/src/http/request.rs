use crate::http::address::Address;
use crate::http::headers::{RequestHeader, RequestHeaderMap};
use crate::http::method::Method;

use std::error::Error;
use std::io::{BufRead, BufReader, Read};
use std::net::SocketAddr;

/// Represents a request to the server.
/// Contains parsed information about the request's data.
#[derive(Clone, Debug)]
pub struct Request {
    /// The method used in making the request, e.g. "GET".
    pub method: Method,
    /// The URI to which the request was made.
    pub uri: String,
    /// The query string of the request.
    pub query: String,
    /// The HTTP version of the request.
    pub version: String,
    /// A map of headers included in the request.
    pub headers: RequestHeaderMap,
    /// The request body, if supplied.
    pub content: Option<Vec<u8>>,
    /// The address from which the request came
    pub address: Address,
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
    pub fn from_stream<T>(stream: &mut T, address: SocketAddr) -> Result<Self, RequestError>
    where
        T: Read,
    {
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
        let version = start_line[2].to_string().replace("\r\n", "");

        let mut uri_iter = start_line[1].splitn(2, '?');
        let uri = uri_iter.next().unwrap().to_string();
        let query = uri_iter.next().unwrap_or("").to_string();

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

        let address = Address::from_headers(&headers, address);

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
                query,
                version,
                headers,
                content: Some(content_buf),
                address,
            })
        } else {
            Ok(Self {
                method,
                uri,
                query,
                version,
                headers,
                content: None,
                address,
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

impl Into<Vec<u8>> for Request {
    fn into(self) -> Vec<u8> {
        let start_line = if self.query.len() == 0 {
            format!("{} {} {}", self.method, self.uri, self.version)
        } else {
            format!(
                "{} {}?{} {}",
                self.method, self.uri, self.query, self.version
            )
        };

        let headers = self
            .headers
            .iter()
            .map(|(k, v)| format!("{}: {}", k.to_string(), v))
            .collect::<Vec<String>>()
            .join("\r\n");

        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend(start_line.as_bytes());
        bytes.extend(b"\r\n");
        bytes.extend(headers.as_bytes());
        bytes.extend(b"\r\n\r\n");

        if let Some(content) = self.content {
            bytes.extend(content);
        }

        bytes
    }
}

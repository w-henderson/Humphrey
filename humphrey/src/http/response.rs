//! Provides functionality for handling HTTP responses.

use crate::http::cookie::SetCookie;
use crate::http::headers::{HeaderLike, HeaderType, Headers};
use crate::http::status::StatusCode;

use std::convert::TryFrom;
use std::error::Error;
use std::io::{BufRead, BufReader, Read};

/// Represents a response from the server.
/// Implements `Into<Vec<u8>>` so can be serialised into bytes to transmit.
///
/// ## Simple Creation
/// ```
/// Response::new(StatusCode::OK, b"Success")
/// ```
///
/// ## Advanced Creation
/// ```
/// Response::empty(StatusCode::OK)
///     .with_bytes(b"Success")
///     .with_header(HeaderType::ContentType, "text/plain")
/// ```
#[derive(Debug)]
pub struct Response {
    /// The HTTP version of the response.
    pub version: String,
    /// The status code of the response, for example 200 OK.
    pub status_code: StatusCode,
    /// A list of the headers included in the response.
    pub headers: Headers,
    /// The body of the response.
    pub body: Vec<u8>,
}

/// An error which occurred during the parsing of a response.
#[derive(Debug, PartialEq, Eq)]
pub enum ResponseError {
    /// The response could not be parsed due to invalid data.
    Response,
    /// The response could not be parsed due to an issue with the stream.
    Stream,
}

impl std::fmt::Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ResponseError")
    }
}

impl Error for ResponseError {}

impl Response {
    /// Creates a new response object with the given status code, bytes and request.
    /// Functionally equivalent to the following (but with some allocation optimisations not shown):
    ///
    /// ```
    /// Response::empty(status_code).with_bytes(bytes)
    /// ```
    ///
    /// ## Note about Headers
    /// If you want to add headers to a response, ideally use `Response::empty` and the builder pattern
    ///   so as to not accidentally override important generated headers such as content length and connection.
    pub fn new<T>(status_code: StatusCode, bytes: T) -> Self
    where
        T: AsRef<[u8]>,
    {
        Self {
            version: "HTTP/1.1".to_string(),
            status_code,
            headers: Headers::new(),
            body: bytes.as_ref().to_vec(),
        }
    }

    /// Creates a new response object with the given status code.
    /// Automatically sets the HTTP version to "HTTP/1.1", sets no headers, and creates an empty body.
    pub fn empty(status_code: StatusCode) -> Self {
        Self {
            version: "HTTP/1.1".to_string(),
            status_code,
            headers: Headers::new(),
            body: Vec::new(),
        }
    }

    /// Creates a redirect response to the given location.
    pub fn redirect<T>(location: T) -> Self
    where
        T: AsRef<str>,
    {
        Self::empty(StatusCode::MovedPermanently).with_header(HeaderType::Location, location)
    }

    /// Adds the given header to the response.
    /// Returns itself for use in a builder pattern.
    pub fn with_header(mut self, header: impl HeaderLike, value: impl AsRef<str>) -> Self {
        self.headers.add(header, value);
        self
    }

    /// Adds the given cookie to the response in the `Set-Cookie` header.
    /// Returns itself for use in a builder pattern.
    ///
    /// ## Example
    /// ```
    /// Response::empty(StatusCode::OK)
    ///     .with_bytes(b"Success")
    ///     .with_cookie(
    ///         SetCookie::new("SessionToken", "abc123")
    ///             .with_max_age(Duration::from_secs(3600))
    ///             .with_secure(true)
    ///             .with_path("/")
    ///     )
    /// ```
    pub fn with_cookie(mut self, cookie: SetCookie) -> Self {
        self.headers.push(cookie.into());
        self
    }

    /// Appends the given bytes to the body.
    /// Returns itself for use in a builder pattern.
    pub fn with_bytes<T>(mut self, bytes: T) -> Self
    where
        T: AsRef<[u8]>,
    {
        self.body.extend(bytes.as_ref());
        self
    }

    /// Returns a reference to the response's headers.
    pub fn get_headers(&self) -> &Headers {
        &self.headers
    }

    /// Returns the body as text, if possible.
    pub fn text(&self) -> Option<String> {
        String::from_utf8(self.body.clone()).ok()
    }

    /// Attempts to read and parse one HTTP response from the given stream.
    ///
    /// Converts chunked transfer encoding into a regular body.
    pub fn from_stream<T>(stream: &mut T) -> Result<Self, ResponseError>
    where
        T: Read,
    {
        let mut reader = BufReader::new(stream);
        let mut start_line_buf: Vec<u8> = Vec::new();
        reader
            .read_until(0xA, &mut start_line_buf)
            .map_err(|_| ResponseError::Stream)?;

        let start_line_string =
            String::from_utf8(start_line_buf).map_err(|_| ResponseError::Response)?;
        let start_line: Vec<&str> = start_line_string.splitn(3, ' ').collect();

        safe_assert(start_line.len() == 3)?;

        let version = start_line[0].to_string();
        let status_code: u16 = start_line[1].parse().map_err(|_| ResponseError::Response)?;
        let status = StatusCode::try_from(status_code).map_err(|_| ResponseError::Response)?;

        let mut headers = Headers::new();

        loop {
            let mut line_buf: Vec<u8> = Vec::new();
            reader
                .read_until(0xA, &mut line_buf)
                .map_err(|_| ResponseError::Stream)?;
            let line = String::from_utf8(line_buf).map_err(|_| ResponseError::Response)?;

            if line == "\r\n" {
                break;
            } else {
                safe_assert(line.len() >= 2)?;
                let line_without_crlf = &line[0..line.len() - 2];
                let line_parts: Vec<&str> = line_without_crlf.splitn(2, ':').collect();
                headers.add(HeaderType::from(line_parts[0]), line_parts[1].trim_start());
            }
        }

        if headers
            .get(&HeaderType::TransferEncoding)
            .and_then(|te| if te == "chunked" { Some(()) } else { None })
            .is_some()
        {
            let mut body: Vec<u8> = Vec::new();

            while let Some(chunk) = parse_chunk(&mut reader) {
                body.extend(chunk);
            }

            headers.remove(&HeaderType::TransferEncoding);
            headers.add(HeaderType::ContentLength, body.len().to_string());

            Ok(Self {
                version,
                status_code: status,
                headers,
                body,
            })
        } else if let Some(content_length) = headers.get(&HeaderType::ContentLength) {
            let content_length: usize = content_length
                .parse()
                .map_err(|_| ResponseError::Response)?;
            let mut content_buf: Vec<u8> = vec![0u8; content_length];
            reader
                .read_exact(&mut content_buf)
                .map_err(|_| ResponseError::Stream)?;

            Ok(Self {
                version,
                status_code: status,
                headers,
                body: content_buf,
            })
        } else {
            Ok(Self {
                version,
                status_code: status,
                headers,
                body: Vec::new(),
            })
        }
    }
}

impl From<Response> for Vec<u8> {
    fn from(val: Response) -> Self {
        let status_line = format!(
            "{} {} {}",
            val.version,
            Into::<u16>::into(val.status_code),
            Into::<&str>::into(val.status_code)
        );

        let mut bytes: Vec<u8> =
            Vec::with_capacity(status_line.len() + val.body.len() + val.headers.len() * 32);
        bytes.extend(status_line.as_bytes());

        for header in val.get_headers().iter() {
            bytes.extend(b"\r\n");
            bytes.extend(header.name.to_string().as_bytes());
            bytes.extend(b": ");
            bytes.extend(header.value.as_bytes());
        }

        bytes.extend(b"\r\n\r\n");

        if !val.body.is_empty() {
            bytes.extend(val.body);
            bytes.extend(b"\r\n");
        }

        bytes
    }
}

/// Parses a chunk using the chunked transfer encoding.
fn parse_chunk<T>(stream: &mut BufReader<T>) -> Option<Vec<u8>>
where
    T: Read,
{
    let mut length_line_buf: Vec<u8> = Vec::new();
    stream.read_until(0xA, &mut length_line_buf).ok()?;
    let length: usize =
        usize::from_str_radix(std::str::from_utf8(&length_line_buf).ok()?.trim_end(), 16).ok()?;

    if length == 0 {
        stream.read_exact(&mut [0u8, 0]).ok()?;
        None
    } else {
        let mut content_buf: Vec<u8> = vec![0u8; length];
        stream.read_exact(&mut content_buf).ok()?;
        stream.read_exact(&mut [0u8, 0]).ok()?;
        Some(content_buf)
    }
}

/// Asserts that the condition is true, returning a `Result`.
fn safe_assert(condition: bool) -> Result<(), ResponseError> {
    match condition {
        true => Ok(()),
        false => Err(ResponseError::Response),
    }
}

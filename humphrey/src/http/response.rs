use crate::http::date::DateTime;
use crate::http::headers::{RequestHeader, ResponseHeader, ResponseHeaderMap};
use crate::http::request::Request;
use crate::http::status::StatusCode;
use std::collections::btree_map::Entry;
use std::convert::TryFrom;
use std::error::Error;
use std::io::{BufRead, BufReader, Read};

/// Represents a response from the server.
/// Implements `Into<Vec<u8>>` so can be serialised into bytes to transmit.
///
/// ## Simple Creation
/// ```
/// Response::new(StatusCode::OK, b"Success", &request)
/// ```
///
/// ## Advanced Creation
/// ```
/// Response::empty(StatusCode::OK)
///     .with_bytes(b"Success")
///     .with_header(ResponseHeader::ContentType, "text/plain".into())
///     .with_request_compatibility(&request)
///     .with_generated_headers()
/// ```
#[derive(Debug)]
pub struct Response {
    /// The HTTP version of the response.
    pub version: String,
    /// The status code of the response, for example 200 OK.
    pub status_code: StatusCode,
    /// A map of the headers included in the response.
    pub headers: ResponseHeaderMap,
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
    /// Response::empty(status_code)
    ///     .with_bytes(bytes)
    ///     .with_request_compatibility(request)
    ///     .with_generated_headers()
    /// ```
    ///
    /// ## Note about Headers
    /// If you want to add headers to a response, ideally use `Response::empty` and the builder pattern
    ///   so as to not accidentally override important generated headers such as content length and connection.
    pub fn new<T>(status_code: StatusCode, bytes: T, request: &Request) -> Self
    where
        T: AsRef<[u8]>,
    {
        Self {
            version: "HTTP/1.1".to_string(),
            status_code,
            headers: ResponseHeaderMap::new(),
            body: bytes.as_ref().to_vec(),
        }
        .with_request_compatibility(request)
        .with_generated_headers()
    }

    /// Creates a new response object with the given status code.
    /// Automatically sets the HTTP version to "HTTP/1.1", sets no headers, and creates an empty body.
    pub fn empty(status_code: StatusCode) -> Self {
        Self {
            version: "HTTP/1.1".to_string(),
            status_code,
            headers: ResponseHeaderMap::new(),
            body: Vec::new(),
        }
    }

    /// Creates a redirect response to the given location.
    pub fn redirect<T>(location: T, request: &Request) -> Self
    where
        T: AsRef<str>,
    {
        Self::empty(StatusCode::MovedPermanently)
            .with_header(ResponseHeader::Location, location.as_ref().to_string())
            .with_request_compatibility(request)
            .with_generated_headers()
    }

    /// Adds the given header to the response.
    /// Returns itself for use in a builder pattern.
    pub fn with_header(mut self, header: ResponseHeader, value: String) -> Self {
        self.headers.insert(header, value);
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

    /// Ensures compatibility with the request it is responding to.
    /// This is done by respecting the `Connection` header of the request (if supplied), as well as
    ///   setting the HTTP version to that of the request. While this is not required, it is
    ///   strongly recommended to fully comply with the browser's request.
    ///
    /// Returns itself for use in a builder pattern.
    pub fn with_request_compatibility(mut self, request: &Request) -> Self {
        if let Some(connection) = request.headers.get(&RequestHeader::Connection) {
            self.headers
                .insert(ResponseHeader::Connection, connection.to_string());
        } else {
            self.headers
                .insert(ResponseHeader::Connection, "Close".to_string());
        }

        self.version = request.version.clone();

        self
    }

    /// Automatically generates required headers.
    /// **This is required** and must be called after calling `.with_bytes(body)` so the content length is accurate.
    /// It will not overwrite any already applied headers.
    ///
    /// The generated headers are:
    /// - `Content-Length`: the calculated content length of the body if non-zero
    /// - `Server`: the server which responded to the request, in this case the string "Humphrey"
    /// - `Date`: the formatted date when the response was created
    /// - `Connection`: will be set to `Close` unless previously set, for example in `.with_request_compatibility(request)`
    ///
    /// Returns itself for use in a builder pattern.
    pub fn with_generated_headers(mut self) -> Self {
        match self.headers.entry(ResponseHeader::Server) {
            Entry::Occupied(_) => (),
            Entry::Vacant(v) => {
                v.insert("Humphrey".to_string());
            }
        }

        match self.headers.entry(ResponseHeader::Date) {
            Entry::Occupied(_) => (),
            Entry::Vacant(v) => {
                v.insert(DateTime::now().to_string());
            }
        }

        match self.headers.entry(ResponseHeader::Connection) {
            Entry::Occupied(_) => (),
            Entry::Vacant(v) => {
                v.insert("Close".to_string());
            }
        }

        if !self.body.is_empty() {
            match self.headers.entry(ResponseHeader::ContentLength) {
                Entry::Occupied(_) => (),
                Entry::Vacant(v) => {
                    v.insert(self.body.len().to_string());
                }
            }
        }

        self
    }

    /// Returns a reference to the response's headers.
    pub fn get_headers(&self) -> &ResponseHeaderMap {
        &self.headers
    }

    /// Attemps to read and parse one HTTP response from the given stream.
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

        let mut headers = ResponseHeaderMap::new();

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
                headers.insert(
                    ResponseHeader::from(line_parts[0]),
                    line_parts[1].trim_start().to_string(),
                );
            }
        }

        if let Some(content_length) = headers.get(&ResponseHeader::ContentLength) {
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
            Into::<u16>::into(val.status_code.clone()),
            Into::<&str>::into(val.status_code.clone())
        );

        let mut bytes: Vec<u8> =
            Vec::with_capacity(status_line.len() + val.body.len() + val.headers.len() * 32);
        bytes.extend(status_line.as_bytes());

        for (header, value) in val.headers {
            bytes.extend(b"\r\n");
            bytes.extend(header.to_string().as_bytes());
            bytes.extend(b": ");
            bytes.extend(value.as_bytes());
        }

        bytes.extend(b"\r\n\r\n");

        if !val.body.is_empty() {
            bytes.extend(val.body);
            bytes.extend(b"\r\n");
        }

        bytes
    }
}

/// Asserts that the condition is true, returning a `Result`.
fn safe_assert(condition: bool) -> Result<(), ResponseError> {
    match condition {
        true => Ok(()),
        false => Err(ResponseError::Response),
    }
}

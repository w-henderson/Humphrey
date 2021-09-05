use crate::http::date::DateTime;
use crate::http::headers::{RequestHeader, ResponseHeader, ResponseHeaderMap};
use crate::http::request::Request;
use crate::http::status::StatusCode;
use std::collections::btree_map::Entry;

/// Represents a response from the server.
/// Implements `Into<Vec<u8>>` so can be serialised into bytes to transmit.
///
/// ## Example
/// ```
/// fn handler(request: &Request, _: Arc<Mutex<()>>) -> Response {
///     Response::new(StatusCode::OK) // create the response
///         .with_bytes(b"<html><body><h1>Success</h1></body></html>".to_vec()) // add the body
///         .with_request_compatibility(request) // ensure compatibility with the request
///         .with_generated_headers() // generate required headers
/// }
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

impl Response {
    /// Creates a new response object with the given status code.
    /// Automatically sets the HTTP version to "HTTP/1.1", sets no headers, and creates an empty body.
    pub fn new(status_code: StatusCode) -> Self {
        Self {
            version: "HTTP/1.1".to_string(),
            status_code,
            headers: ResponseHeaderMap::new(),
            body: Vec::new(),
        }
    }

    /// Adds the given header to the response.
    /// Returns itself for use in a builder pattern.
    pub fn with_header(mut self, header: ResponseHeader, value: String) -> Self {
        self.headers.insert(header, value);
        self
    }
    /// Appends the given bytes to the body.
    /// Returns itself for use in a builder pattern.
    pub fn with_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.body.extend(bytes);
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

        if self.body.len() > 0 {
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
}

impl Into<Vec<u8>> for Response {
    fn into(self) -> Vec<u8> {
        let status_line = format!(
            "{} {} {}",
            self.version,
            Into::<u16>::into(self.status_code.clone()),
            Into::<&str>::into(self.status_code.clone())
        );

        let mut bytes: Vec<u8> = Vec::with_capacity(status_line.len());
        bytes.extend(status_line.as_bytes());

        for (header, value) in self.headers {
            bytes.extend(b"\r\n");
            bytes.extend(header.to_string().as_bytes());
            bytes.extend(b": ");
            bytes.extend(value.as_bytes());
        }

        bytes.extend(b"\r\n\r\n");

        if self.body.len() != 0 {
            bytes.extend(self.body);
            bytes.extend(b"\r\n");
        }

        bytes
    }
}

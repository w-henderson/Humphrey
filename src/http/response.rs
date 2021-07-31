use crate::http::date::HTTPDate;
use crate::http::headers::{RequestHeader, ResponseHeader, ResponseHeaderMap};
use crate::http::request::Request;
use crate::http::status::StatusCode;
use std::collections::btree_map::Entry;

#[derive(Debug)]
pub struct Response {
    version: String,
    status_code: StatusCode,
    headers: ResponseHeaderMap,
    body: Vec<u8>,
}

impl Response {
    pub fn new(status_code: StatusCode) -> Self {
        Self {
            version: "HTTP/1.1".to_string(),
            status_code,
            headers: ResponseHeaderMap::new(),
            body: Vec::new(),
        }
    }

    pub fn with_header(mut self, header: ResponseHeader, value: String) -> Self {
        self.headers.insert(header, value);
        self
    }

    pub fn with_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.body.extend(bytes);
        self
    }

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

    pub fn with_generated_headers(mut self) -> Self {
        match self.headers.entry(ResponseHeader::ContentLength) {
            Entry::Occupied(_) => (),
            Entry::Vacant(v) => {
                v.insert(self.body.len().to_string());
            }
        }

        match self.headers.entry(ResponseHeader::Server) {
            Entry::Occupied(_) => (),
            Entry::Vacant(v) => {
                v.insert("Humphrey".to_string());
            }
        }

        match self.headers.entry(ResponseHeader::Date) {
            Entry::Occupied(_) => (),
            Entry::Vacant(v) => {
                v.insert(HTTPDate::now());
            }
        }

        match self.headers.entry(ResponseHeader::Connection) {
            Entry::Occupied(_) => (),
            Entry::Vacant(v) => {
                v.insert("Close".to_string());
            }
        }

        self
    }

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
            bytes.extend(String::from(header).as_bytes());
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

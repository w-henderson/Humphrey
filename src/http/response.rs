use crate::http::date::HTTPDate;
use crate::http::headers::{ResponseHeader, ResponseHeaderMap};
use crate::http::status::StatusCode;
use std::collections::btree_map::Entry;

pub struct Response {
    status_code: StatusCode,
    headers: ResponseHeaderMap,
    body: Vec<u8>,
}

impl Response {
    pub fn new(status_code: StatusCode) -> Self {
        Self {
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

        self
    }

    pub fn get_headers(&self) -> &ResponseHeaderMap {
        &self.headers
    }
}

impl Into<Vec<u8>> for Response {
    fn into(self) -> Vec<u8> {
        let status_line = format!(
            "HTTP/1.1 {} {}",
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
        bytes.extend(self.body);

        bytes
    }
}

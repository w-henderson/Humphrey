use crate::http::date::HTTPDate;
use crate::http::headers::{ResponseHeader, ResponseHeaderMap};
use crate::http::status::StatusCode;

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

    pub fn add_header(&mut self, header: ResponseHeader, value: String) -> &mut Self {
        self.headers.insert(header, value);
        self
    }

    pub fn add_bytes(&mut self, bytes: Vec<u8>) -> &mut Self {
        self.body.extend(bytes);
        self
    }

    pub fn generate_headers(&mut self) -> &mut Self {
        self.add_header(ResponseHeader::ContentLength, self.body.len().to_string())
            .add_header(ResponseHeader::Server, "Humphrey".to_string())
            .add_header(ResponseHeader::Date, HTTPDate::now())
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

#![allow(dead_code, unused_imports)]
use crate::http::address::Address;
use crate::http::headers::RequestHeader;
use crate::http::method::Method;
use crate::http::Request;
use std::{
    collections::BTreeMap,
    io::Read,
    net::{SocketAddr, ToSocketAddrs},
};

struct MockStream {
    data: Vec<u8>,
}

impl MockStream {
    fn with_data(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl Read for MockStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut index = 0;
        for byte in buf {
            *byte = self.data[index];
            index += 1;
            if index == self.data.len() {
                break;
            }
        }

        std::io::Result::Ok(self.data.len())
    }
}

#[test]
fn test_request_from_stream() {
    let test_data = b"GET /testpath?foo=bar HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let mut stream = MockStream::with_data(test_data.to_vec());
    let request = Request::from_stream(&mut stream, "1.2.3.4:5678".parse().unwrap());

    assert!(request.is_ok());

    let request = request.unwrap();
    let expected_uri: String = "/testpath".into();
    let expected_query: String = "foo=bar".into();
    assert_eq!(request.method, Method::Get);
    assert_eq!(request.uri, expected_uri);
    assert_eq!(request.query, expected_query);
    assert_eq!(request.version, "HTTP/1.1");
    assert_eq!(request.content, None);
    assert_eq!(request.address, Address::new("1.2.3.4:5678"));

    let mut expected_headers: BTreeMap<RequestHeader, String> = BTreeMap::new();
    expected_headers.insert(RequestHeader::Host, "localhost".to_string());
    assert_eq!(request.headers, expected_headers);
}

#[test]
fn test_bytes_from_request() {
    let mut test_data = Request {
        method: Method::Get,
        uri: "/test".into(),
        query: "foo=bar".into(),
        version: "HTTP/1.1".into(),
        headers: BTreeMap::new(),
        content: Some(b"this is a test".to_vec()),
        address: Address::new("1.2.3.4:5678"),
    };

    test_data
        .headers
        .insert(RequestHeader::ContentLength, "14".into());
    test_data
        .headers
        .insert(RequestHeader::ContentType, "text/plain".into());

    let expected_bytes = b"GET /test?foo=bar HTTP/1.1\r\nContent-Length: 14\r\nContent-Type: text/plain\r\n\r\nthis is a test".to_vec();

    let bytes: Vec<u8> = test_data.into();

    assert_eq!(bytes, expected_bytes);
}

#[test]
fn test_proxied_request_from_stream() {
    let test_data =
        b"GET /testpath HTTP/1.1\r\nHost: localhost\r\nX-Forwarded-For: 9.10.11.12,13.14.15.16\r\n\r\n";
    let mut stream = MockStream::with_data(test_data.to_vec());
    let request = Request::from_stream(&mut stream, "1.2.3.4:5678".parse().unwrap());

    assert!(request.is_ok());

    let request = request.unwrap();
    let expected_uri: String = "/testpath".into();
    assert_eq!(request.method, Method::Get);
    assert_eq!(request.uri, expected_uri);
    assert_eq!(request.version, "HTTP/1.1");
    assert_eq!(request.content, None);
    assert_eq!(
        request.address,
        Address {
            origin_addr: "13.14.15.16".parse().unwrap(),
            proxies: vec!["9.10.11.12".parse().unwrap(), "1.2.3.4".parse().unwrap()],
            port: 5678
        }
    );

    let mut expected_headers: BTreeMap<RequestHeader, String> = BTreeMap::new();
    expected_headers.insert(RequestHeader::Host, "localhost".to_string());
    expected_headers.insert(
        RequestHeader::Custom {
            name: "x-forwarded-for".into(),
        },
        "9.10.11.12,13.14.15.16".to_string(),
    );
    assert_eq!(request.headers, expected_headers);
}

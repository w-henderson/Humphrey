#![allow(dead_code, unused_imports)]
use crate::http::address::Address;
use crate::http::cookie::Cookie;
use crate::http::headers::{Header, HeaderType, Headers};
use crate::http::method::Method;
use crate::http::Request;
use crate::tests::mock_stream::MockStream;

use std::collections::{BTreeMap, VecDeque};
use std::io::Read;
use std::iter::FromIterator;
use std::net::{SocketAddr, ToSocketAddrs};

#[test]
fn test_request_from_stream() {
    let test_data = b"GET /testpath?foo=bar HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let mut stream = MockStream::with_data(VecDeque::from_iter(test_data.iter().cloned()));
    let request = Request::from_stream(&mut stream, "1.2.3.4:5678".parse().unwrap());

    let request = request.unwrap();
    let expected_uri: String = "/testpath".into();
    let expected_query: String = "foo=bar".into();
    assert_eq!(request.method, Method::Get);
    assert_eq!(request.uri, expected_uri);
    assert_eq!(request.query, expected_query);
    assert_eq!(request.version, "HTTP/1.1");
    assert_eq!(request.content, None);
    assert_eq!(request.address, Address::new("1.2.3.4:5678").unwrap());

    let mut expected_headers: Headers = Headers::new();
    expected_headers.add(HeaderType::Host, "localhost");
    assert_eq!(request.headers, expected_headers);
}

#[test]
fn test_cookie_request() {
    let test_data = b"GET / HTTP/1.1\r\nHost: localhost\r\nCookie: foo=bar; baz=qux\r\n\r\n";
    let mut stream = MockStream::with_data(VecDeque::from_iter(test_data.iter().cloned()));
    let request = Request::from_stream(&mut stream, "1.2.3.4:5678".parse().unwrap()).unwrap();

    let mut expected_cookies = vec![Cookie::new("foo", "bar"), Cookie::new("baz", "qux")];

    assert_eq!(request.get_cookies(), expected_cookies);

    assert_eq!(request.get_cookie("baz"), expected_cookies.pop());
    assert_eq!(request.get_cookie("foo"), expected_cookies.pop());
    assert_eq!(request.get_cookie("sus"), None);
}

#[test]
fn test_bytes_from_request() {
    let mut test_data = Request {
        method: Method::Get,
        uri: "/test".into(),
        query: "foo=bar".into(),
        version: "HTTP/1.1".into(),
        headers: Headers::new(),
        content: Some(b"this is a test".to_vec()),
        address: Address::new("1.2.3.4:5678").unwrap(),
    };

    test_data.headers.add(HeaderType::ContentLength, "14");
    test_data.headers.add(HeaderType::ContentType, "text/plain");

    let expected_bytes = b"GET /test?foo=bar HTTP/1.1\r\nContent-Length: 14\r\nContent-Type: text/plain\r\n\r\nthis is a test".to_vec();

    let bytes: Vec<u8> = test_data.into();

    assert_eq!(bytes, expected_bytes);
}

#[test]
fn test_proxied_request_from_stream() {
    let test_data =
        b"GET /testpath HTTP/1.1\r\nHost: localhost\r\nX-Forwarded-For: 9.10.11.12,13.14.15.16\r\n\r\n";
    let mut stream = MockStream::with_data(VecDeque::from_iter(test_data.iter().cloned()));
    let request = Request::from_stream(&mut stream, "1.2.3.4:5678".parse().unwrap());

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

    let mut expected_headers: Headers = Headers::new();
    expected_headers.add(HeaderType::Host, "localhost");
    expected_headers.add("X-Forwarded-For", "9.10.11.12,13.14.15.16");

    assert_eq!(request.headers, expected_headers);
}

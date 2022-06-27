use crate::client::{Client, ParsedUrl, Protocol};
use crate::http::headers::{HeaderType, Headers};

use std::net::ToSocketAddrs;

#[test]
fn test_url_parser() {
    let google_ip_80 = "google.com:80".to_socket_addrs().unwrap().next().unwrap();
    let google_ip_443 = "google.com:443".to_socket_addrs().unwrap().next().unwrap();

    let urls = [
        Client::parse_url("https://google.com").unwrap(),
        Client::parse_url("http://google.com").unwrap(),
        Client::parse_url("https://google.com/maps").unwrap(),
        Client::parse_url("https://google.com/search?q=test").unwrap(),
    ];

    let mut expected_host_headers = Headers::new();
    expected_host_headers.add(HeaderType::Host, "google.com");

    let expected_urls = [
        ParsedUrl {
            protocol: Protocol::Https,
            host: google_ip_443,
            path: "/".to_string(),
            query: "".to_string(),
            host_headers: expected_host_headers.clone(),
        },
        ParsedUrl {
            protocol: Protocol::Http,
            host: google_ip_80,
            path: "/".to_string(),
            query: "".to_string(),
            host_headers: expected_host_headers.clone(),
        },
        ParsedUrl {
            protocol: Protocol::Https,
            host: google_ip_443,
            path: "/maps".to_string(),
            query: "".to_string(),
            host_headers: expected_host_headers.clone(),
        },
        ParsedUrl {
            protocol: Protocol::Https,
            host: google_ip_443,
            path: "/search".to_string(),
            query: "q=test".to_string(),
            host_headers: expected_host_headers,
        },
    ];

    for (url, expected_url) in urls.iter().zip(expected_urls.iter()) {
        assert_eq!(url.protocol, expected_url.protocol);
        assert_eq!(url.path, expected_url.path);
        assert_eq!(url.query, expected_url.query);
        assert_eq!(url.host_headers, expected_url.host_headers);
    }
}

#[test]
fn test_content_length() {
    let mut client = Client::new();

    let post_request = client
        .post("http://127.0.0.1/post", b"Hello, world!".to_vec())
        .unwrap()
        .into_inner();
    let put_request = client
        .put("http://127.0.0.1/put", b"Hello, world!".to_vec())
        .unwrap()
        .into_inner();
    let empty_request = client
        .post("http://127.0.0.1/post", Vec::new())
        .unwrap()
        .into_inner();

    assert_eq!(post_request.headers.get("Content-Length"), Some("13"));
    assert_eq!(put_request.headers.get("Content-Length"), Some("13"));
    assert_eq!(empty_request.headers.get("Content-Length"), Some("0"));
}

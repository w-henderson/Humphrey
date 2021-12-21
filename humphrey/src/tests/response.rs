#![allow(unused_imports)]
use crate::http::headers::{ResponseHeader, ResponseHeaderMap};
use crate::http::response::Response;
use crate::http::status::StatusCode;
use crate::tests::mock_stream::MockStream;
use std::collections::BTreeMap;

#[test]
fn test_response() {
    let response = Response::empty(StatusCode::OK)
        .with_bytes(b"<body>test</body>")
        .with_header(ResponseHeader::ContentType, "text/html".to_string())
        .with_header(ResponseHeader::ContentLanguage, "en-GB".to_string())
        .with_header(
            ResponseHeader::Date,
            "Thu, 1 Jan 1970 00:00:00 GMT".to_string(),
        ); // this would never be manually set in prod, but is obviously required for testing

    assert!(response
        .get_headers()
        .get(&ResponseHeader::ContentType)
        .is_some());

    assert_eq!(
        response
            .get_headers()
            .get(&ResponseHeader::ContentType)
            .unwrap(),
        "text/html"
    );

    let expected_bytes: Vec<u8> = b"HTTP/1.1 200 OK\r\nDate: Thu, 1 Jan 1970 00:00:00 GMT\r\nContent-Language: en-GB\r\nContent-Type: text/html\r\n\r\n<body>test</body>\r\n".to_vec();
    let bytes: Vec<u8> = response.into();

    assert_eq!(bytes, expected_bytes);
}

#[test]
fn test_response_from_stream() {
    let test_data = b"HTTP/1.1 404 Not Found\r\nContent-Length: 51\r\n\r\nThe requested resource was not found on the server.\r\n";
    let mut stream = MockStream::with_data(test_data.to_vec());
    let response = Response::from_stream(&mut stream);

    assert!(response.is_ok());

    let response = response.unwrap();
    let expected_body = b"The requested resource was not found on the server.".to_vec();
    assert_eq!(response.body, expected_body);
    assert_eq!(response.version, "HTTP/1.1".to_string());
    assert_eq!(response.status_code, StatusCode::NotFound);

    let mut expected_headers: BTreeMap<ResponseHeader, String> = BTreeMap::new();
    expected_headers.insert(ResponseHeader::ContentLength, "51".into());
    assert_eq!(response.headers, expected_headers);
}

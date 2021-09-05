#![allow(unused_imports)]
use crate::http::headers::{ResponseHeader, ResponseHeaderMap};
use crate::http::response::Response;
use crate::http::status::StatusCode;

#[test]
fn test_response() {
    let response = Response::new(StatusCode::OK)
        .with_bytes(b"<body>test</body>".to_vec())
        .with_header(ResponseHeader::ContentType, "text/html".to_string())
        .with_header(ResponseHeader::ContentLanguage, "en-GB".to_string())
        .with_header(
            ResponseHeader::Date,
            "Thu, 1 Jan 1970 00:00:00 GMT".to_string(),
        ) // this would never be manually set in prod, but is obviously required for testing
        .with_generated_headers();

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

    assert_eq!(
        response.get_headers().get(&ResponseHeader::Server).unwrap(),
        "Humphrey"
    );

    let expected_bytes: Vec<u8> = b"HTTP/1.1 200 OK\r\nConnection: Close\r\nDate: Thu, 1 Jan 1970 00:00:00 GMT\r\nServer: Humphrey\r\nContent-Language: en-GB\r\nContent-Length: 17\r\nContent-Type: text/html\r\n\r\n<body>test</body>\r\n".to_vec();
    let bytes: Vec<u8> = response.into();

    assert_eq!(bytes, expected_bytes);
}

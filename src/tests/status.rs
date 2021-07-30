#![allow(unused_imports)]
use crate::http::status::StatusCode;
use std::convert::{TryFrom, TryInto};

#[test]
fn test_from_code() {
    let valid_codes: [u16; 39] = [
        100, 101, 200, 201, 202, 203, 204, 205, 206, 300, 301, 302, 303, 304, 305, 307, 400, 401,
        403, 404, 405, 406, 407, 408, 409, 410, 411, 412, 413, 414, 415, 416, 417, 500, 501, 502,
        503, 504, 505,
    ];

    for code in valid_codes {
        assert!(StatusCode::try_from(code).is_ok());
    }

    assert!(StatusCode::try_from(69).is_err());
    assert!(StatusCode::try_from(420).is_err());
    assert!(StatusCode::try_from(1337).is_err());
}

#[test]
fn test_into_code() {
    assert!(TryInto::<StatusCode>::try_into(200u16).is_ok());
    assert!(TryInto::<StatusCode>::try_into(404u16).is_ok());
    assert!(TryInto::<StatusCode>::try_into(1337u16).is_err());
}

#[test]
fn test_into_string() {
    assert_eq!(Into::<&str>::into(StatusCode::OK), "OK");
    assert_eq!(Into::<&str>::into(StatusCode::NotFound), "Not Found");
    assert_eq!(Into::<&str>::into(StatusCode::BadGateway), "Bad Gateway");
}

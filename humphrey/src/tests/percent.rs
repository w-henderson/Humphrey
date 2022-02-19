use crate::percent::{PercentDecode, PercentEncode};

#[test]
fn encode_unreserved_chars() {
    let string = "thisisatest";
    let encoded = string.percent_encode();

    assert_eq!(encoded, string);
}

#[test]
fn encode_reserved_chars() {
    let string = "this is a test! (and brackets)";
    let encoded = string.percent_encode();

    assert_eq!(encoded, "this%20is%20a%20test%21%20%28and%20brackets%29");
}

#[test]
fn encode_bytes() {
    let bytes = b"this is a \0null character";
    let encoded = bytes.percent_encode();

    assert_eq!(encoded, "this%20is%20a%20%00null%20character");
}

#[test]
fn decode_unreserved_chars() {
    let string = "thisisatest";
    let decoded = string.percent_decode();

    assert_eq!(decoded, Some(string.as_bytes().to_vec()));
}

#[test]
fn decode_reserved_chars() {
    let string = "this%20is%20a%20test%21%20%28and%20brackets%29";
    let decoded = string.percent_decode();

    assert_eq!(decoded, Some(b"this is a test! (and brackets)".to_vec()));
}

#[test]
fn decode_bytes() {
    let string = "this%20is%20a%20%00null%20character";
    let decoded = string.percent_decode();

    assert_eq!(decoded, Some(b"this is a \0null character".to_vec()));
}

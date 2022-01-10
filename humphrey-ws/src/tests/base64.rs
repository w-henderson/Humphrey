use crate::util::base64::{Base64Decode, Base64Encode};

#[test]
fn test_base64_encode() {
    let padding_0_input = "foo";
    let padding_0_expected = "Zm9v";
    let padding_0_result = padding_0_input.encode();
    assert_eq!(padding_0_result, padding_0_expected);

    let padding_1_input = "yeet";
    let padding_1_expected = "eWVldA==";
    let padding_1_result = padding_1_input.encode();
    assert_eq!(padding_1_result, padding_1_expected);

    let padding_2_input = "hello";
    let padding_2_expected = "aGVsbG8=";
    let padding_2_result = padding_2_input.encode();
    assert_eq!(padding_2_result, padding_2_expected);
}

#[test]
fn test_base64_decode() {
    let padding_0_input = "Zm9v";
    let padding_0_expected = b"foo";
    let padding_0_result = padding_0_input.decode().unwrap();
    assert_eq!(padding_0_result, padding_0_expected);

    let padding_1_input = "eWVldA==";
    let padding_1_expected = b"yeet";
    let padding_1_result = padding_1_input.decode().unwrap();
    assert_eq!(padding_1_result, padding_1_expected);

    let padding_2_input = "aGVsbG8=";
    let padding_2_expected = b"hello";
    let padding_2_result = padding_2_input.decode().unwrap();
    assert_eq!(padding_2_result, padding_2_expected);
}

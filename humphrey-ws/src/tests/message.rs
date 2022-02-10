use crate::message::Message;
use crate::tests::frame::{FRAME_1_BYTES, FRAME_2_BYTES, STANDALONE_FRAME_BYTES, UNMASKED_BYTES};
use crate::tests::mock_stream::MockStream;

#[test]
fn test_read_single_message() {
    let mut stream = MockStream::with_data(STANDALONE_FRAME_BYTES.to_vec());
    let message = Message::from_stream(&mut stream).unwrap();
    let expected_message = Message::new("hello");
    assert_eq!(message.bytes(), expected_message.bytes());
}

#[test]
fn test_read_fragmented_message() {
    let mut bytes = Vec::with_capacity(23);
    bytes.extend(FRAME_1_BYTES);
    bytes.extend(FRAME_2_BYTES);

    let mut stream = MockStream::with_data(bytes);
    let message = Message::from_stream(&mut stream).unwrap();
    let expected_message = Message::new("hello world");

    assert_eq!(message.bytes(), expected_message.bytes());
}

#[test]
fn test_write_message() {
    let message = Message::new("hello world");
    let bytes = message.to_frame();

    assert_eq!(bytes, UNMASKED_BYTES);
}

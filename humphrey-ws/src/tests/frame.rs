#![allow(clippy::unusual_byte_groupings)]

use crate::frame::{Frame, Opcode};
use crate::tests::mock_stream::MockStream;

#[rustfmt::skip]
pub const FRAME_1_BYTES: [u8; 12] = [
    0b0000_0001, // not fin, opcode text
    0b1_0000110, // mask, payload length 6
    0x69, 0x69, 0x69, 0x69, // masking key 0x69696969
    1, 12, 5, 5, 6, 73 // masked payload "hello "
];

#[rustfmt::skip]
pub const FRAME_2_BYTES: [u8; 11] = [
    0b1000_0000, // fin, opcode continuation
    0b1_0000101, // mask, payload length 5
    0x69, 0x69, 0x69, 0x69, // masking key 0x69696969
    30, 6, 27, 5, 13 // masked payload "world"
];

#[rustfmt::skip]
pub const STANDALONE_FRAME_BYTES: [u8; 11] = [
    0b1000_0001, // fin, opcode text
    0b1_0000101, // mask, payload length 5
    0x69, 0x69, 0x69, 0x69, // masking key 0x69696969
    1, 12, 5, 5, 6 // masked payload "hello"
];

#[rustfmt::skip]
pub const UNMASKED_BYTES: [u8; 13] = [
    0b1000_0001, // fin, opcode text
    0b0_0001011, // not mask, payload length 11
    b'h', b'e', b'l', b'l', b'o', b' ', b'w', b'o', b'r', b'l', b'd' // unmasked payload "hello world"
];

#[rustfmt::skip]
pub const MEDIUM_FRAME_BYTES: [u8; 8] = [
    0b1000_0001, // fin, opcode text
    0b1_1111110, // mask, payload length 126 (extended payload length 16 bit)
    0x01, 0x00, // extended payload length of 256
    0x69, 0x69, 0x69, 0x69, // masking key 0x69696969
];

#[rustfmt::skip]
pub const LONG_FRAME_BYTES: [u8; 14] = [
    0b1000_0001, // fin, opcode text
    0b1_1111111, // mask, payload length 127 (extended payload length 64 bit)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, // extended payload length of 65536
    0x69, 0x69, 0x69, 0x69, // masking key 0x69696969
];

#[test]
fn test_initial_frame() {
    let mut bytes = Vec::with_capacity(23);
    bytes.extend(FRAME_1_BYTES);
    bytes.extend(FRAME_2_BYTES);

    let mut stream = MockStream::with_data(bytes);
    let frame = Frame::from_stream(&mut stream).unwrap();

    let expected_frame = Frame {
        fin: false,
        rsv: [false; 3],
        opcode: Opcode::Text,
        mask: true,
        masking_key: [0x69; 4],
        length: 6,
        payload: b"hello ".to_vec(),
    };

    assert_eq!(frame, expected_frame);
}

#[test]
fn test_continuation_frame() {
    let mut stream = MockStream::with_data(FRAME_2_BYTES.to_vec());
    let frame = Frame::from_stream(&mut stream).unwrap();

    let expected_frame = Frame {
        fin: true,
        rsv: [false; 3],
        opcode: Opcode::Continuation,
        mask: true,
        masking_key: [0x69; 4],
        length: 5,
        payload: b"world".to_vec(),
    };

    assert_eq!(frame, expected_frame);
}

#[test]
fn test_standalone_frame() {
    let mut stream = MockStream::with_data(STANDALONE_FRAME_BYTES.to_vec());
    let frame = Frame::from_stream(&mut stream).unwrap();

    let expected_frame = Frame {
        fin: true,
        rsv: [false; 3],
        opcode: Opcode::Text,
        mask: true,
        masking_key: [0x69; 4],
        length: 5,
        payload: b"hello".to_vec(),
    };

    assert_eq!(frame, expected_frame);
}

#[test]
fn test_medium_frame() {
    let mut bytes = Vec::with_capacity(264);
    bytes.extend(MEDIUM_FRAME_BYTES);
    bytes.extend(vec![b'x' ^ 0x69; 256]);

    let mut stream = MockStream::with_data(bytes);
    let frame = Frame::from_stream(&mut stream).unwrap();

    let expected_frame = Frame {
        fin: true,
        rsv: [false; 3],
        opcode: Opcode::Text,
        mask: true,
        masking_key: [0x69; 4],
        length: 256,
        payload: vec![b'x'; 256],
    };

    assert_eq!(frame, expected_frame);
}

#[test]
fn test_long_frame() {
    let mut bytes = Vec::with_capacity(65550);
    bytes.extend(LONG_FRAME_BYTES);
    bytes.extend(vec![b'x' ^ 0x69; 65536]);

    let mut stream = MockStream::with_data(bytes);
    let frame = Frame::from_stream(&mut stream).unwrap();

    let expected_frame = Frame {
        fin: true,
        rsv: [false; 3],
        opcode: Opcode::Text,
        mask: true,
        masking_key: [0x69; 4],
        length: 65536,
        payload: vec![b'x'; 65536],
    };

    assert_eq!(frame, expected_frame);
}

#[test]
fn test_write() {
    let frame = Frame {
        fin: true,
        rsv: [false; 3],
        opcode: Opcode::Text,
        mask: false,
        masking_key: [0; 4],
        length: 11,
        payload: b"hello world".to_vec(),
    };

    let bytes: Vec<u8> = frame.into();

    assert_eq!(bytes, UNMASKED_BYTES.to_vec());
}

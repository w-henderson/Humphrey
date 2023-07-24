#![allow(dead_code)]

use std::collections::VecDeque;
use std::io::Read;

#[cfg(feature = "tokio")]
use tokio::io::AsyncRead;

pub struct MockStream {
    data: VecDeque<u8>,
}

impl MockStream {
    pub fn with_data(data: VecDeque<u8>) -> Self {
        Self { data }
    }
}

impl Read for MockStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut bytes_written: usize = 0;

        for byte in buf {
            if let Some(new_byte) = self.data.pop_front() {
                *byte = new_byte;
                bytes_written += 1;
            } else {
                return std::io::Result::Ok(bytes_written);
            }
        }

        std::io::Result::Ok(bytes_written)
    }
}

#[cfg(feature = "tokio")]
impl AsyncRead for MockStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let mut bytes_written: usize = 0;

        for byte in buf.initialize_unfilled() {
            if let Some(new_byte) = self.data.pop_front() {
                *byte = new_byte;
                bytes_written += 1;
            } else {
                buf.advance(bytes_written);
                return std::task::Poll::Ready(std::io::Result::Ok(()));
            }
        }

        buf.advance(bytes_written);
        std::task::Poll::Ready(std::io::Result::Ok(()))
    }
}

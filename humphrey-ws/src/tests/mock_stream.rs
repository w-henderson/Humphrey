#![allow(dead_code)]

use std::io::{Read, Write};

pub struct MockStream {
    data: Vec<u8>,
    index: usize,
}

impl MockStream {
    pub fn with_data(data: Vec<u8>) -> Self {
        Self { data, index: 0 }
    }
}

impl Read for MockStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let initial_index = self.index;
        for byte in buf {
            *byte = self.data[self.index];
            self.index += 1;
            if self.index == self.data.len() {
                break;
            }
        }

        std::io::Result::Ok(self.index - initial_index)
    }
}

impl Write for MockStream {
    fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
        todo!()
    }

    fn flush(&mut self) -> std::io::Result<()> {
        todo!()
    }
}

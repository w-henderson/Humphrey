#![allow(dead_code)]

use std::io::Read;

pub struct MockStream {
    data: Vec<u8>,
}

impl MockStream {
    pub fn with_data(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl Read for MockStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut index = 0;
        for byte in buf {
            *byte = self.data[index];
            index += 1;
            if index == self.data.len() {
                break;
            }
        }

        std::io::Result::Ok(self.data.len())
    }
}

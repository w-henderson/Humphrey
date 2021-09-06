use std::collections::HashMap;

pub trait Params {
    fn encode(&self) -> Vec<u8>;
}

impl Params for HashMap<String, String> {
    fn encode(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();

        for (name, value) in self.iter() {
            if name.len() < 128 {
                result.push(name.len() as u8);
            } else {
                result.extend((name.len() as u32).to_be_bytes());
            }

            if value.len() < 128 {
                result.push(value.len() as u8);
            } else {
                result.extend((value.len() as u32).to_be_bytes());
            }

            result.extend(name.as_bytes());
            result.extend(value.as_bytes());
        }

        result
    }
}

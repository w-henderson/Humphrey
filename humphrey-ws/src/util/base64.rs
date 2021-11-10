const ALPHABET: [u8; 64] = *b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub trait Base64Encode {
    fn encode(&self) -> String;
}

pub trait Base64Decode {
    fn decode(&self) -> Result<Vec<u8>, ()>;
}

impl<T> Base64Encode for T
where
    T: AsRef<[u8]>,
{
    fn encode(&self) -> String {
        let bytes = self.as_ref();
        let mut result = String::with_capacity((bytes.len() * 4) / 3 + 4);

        for group_index in 0..bytes.len() / 3 {
            let group = &bytes[group_index * 3..group_index * 3 + 3];

            result.push(ALPHABET[(group[0] >> 2) as usize] as char);
            result.push(ALPHABET[((group[0] & 0x03) << 4 | group[1] >> 4) as usize] as char);
            result.push(ALPHABET[((group[1] & 0x0f) << 2 | group[2] >> 6) as usize] as char);
            result.push(ALPHABET[group[2] as usize & 0x3f] as char);
        }

        let remaining = bytes.len() % 3;
        let group = &bytes[(bytes.len() - remaining)..];
        if remaining == 1 {
            result.push(ALPHABET[(group[0] >> 2) as usize] as char);
            result.push(ALPHABET[((group[0] & 0x03) << 4) as usize] as char);
            result.push('=');
            result.push('=');
        } else if remaining == 2 {
            result.push(ALPHABET[(group[0] >> 2) as usize] as char);
            result.push(ALPHABET[((group[0] & 0x03) << 4 | group[1] >> 4) as usize] as char);
            result.push(ALPHABET[((group[1] & 0x0f) << 2) as usize] as char);
            result.push('=');
        }

        result
    }
}

impl<T> Base64Decode for T
where
    T: AsRef<str>,
{
    fn decode(&self) -> Result<Vec<u8>, ()> {
        let input = self.as_ref();
        let mut result: Vec<u8> = Vec::with_capacity(input.len() * 3 / 4);

        for group in input.as_bytes().chunks(4) {
            let mut decoded: u32 = 0;
            let mut broken: usize = 4;

            for (i, tem) in group.iter().enumerate() {
                match tem {
                    b'A'..=b'Z' => decoded |= ((tem - b'A') as u32) << (6 * (3 - i)),
                    b'a'..=b'z' => decoded |= ((tem - b'a' + 26) as u32) << (6 * (3 - i)),
                    b'0'..=b'9' => decoded |= ((tem - b'0' + 52) as u32) << (6 * (3 - i)),
                    b'+' => decoded |= 62_u32 << (6 * i),
                    b'/' => decoded |= 63_u32 << (6 * i),
                    b'=' => {
                        broken = i;
                        break;
                    }
                    _ => return Err(()),
                }
            }

            result.extend_from_slice(&decoded.to_be_bytes()[1..broken]);
        }

        Ok(result)
    }
}

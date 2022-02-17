//! Provides percent-encoding functionality.

const UNRESERVED_CHARACTERS: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.~";

/// A trait which represents the ability of a type to be percent-encoded.
pub trait PercentEncode {
    /// Percent-encode the value.
    fn percent_encode(&self) -> String;
}

/// A trait which represents the ability of a type to be percent-decoded.
pub trait PercentDecode {
    /// Attempt to percent-decode the value.
    fn percent_decode(&self) -> Option<Vec<u8>>;
}

impl<T> PercentEncode for T
where
    T: AsRef<[u8]>,
{
    fn percent_encode(&self) -> String {
        let bytes = self.as_ref();
        let mut encoded = String::with_capacity(bytes.len() * 3);

        for byte in bytes {
            if UNRESERVED_CHARACTERS.contains(byte) {
                encoded.push(*byte as char);
            } else {
                encoded += &format!("%{:02X}", byte);
            }
        }

        encoded
    }
}

impl<T> PercentDecode for T
where
    T: AsRef<str>,
{
    fn percent_decode(&self) -> Option<Vec<u8>> {
        let length = self.as_ref().len();
        let mut chars = self.as_ref().bytes();
        let mut decoded = Vec::with_capacity(length);

        while let Some(character) = chars.next() {
            if character == b'%' {
                let [hex_dig_1, hex_dig_2] = [chars.next()?, chars.next()?];
                let hex = format!("{}{}", hex_dig_1 as char, hex_dig_2 as char);
                let byte = u8::from_str_radix(&hex, 16).ok()?;
                decoded.push(byte);
            } else {
                decoded.push(character);
            }
        }

        Some(decoded)
    }
}

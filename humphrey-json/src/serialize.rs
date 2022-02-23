//! Provides JSON serialization functionality.

use crate::Value;

impl Value {
    /// Serialize a JSON value into a string.
    pub fn serialize(&self) -> String {
        match self {
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => string_to_string(s),
            Value::Array(a) => array_to_string(a),
            Value::Object(o) => object_to_string(o),
        }
    }
}

fn string_to_string(s: &str) -> String {
    let mut string = String::with_capacity(s.len() + 2);

    string.push('"');

    for c in s.chars() {
        match c as u32 {
            0x22 => string.push_str("\\\""),
            0x5c => string.push_str("\\\\"),
            0x2f => string.push_str("\\/"),
            0x08 => string.push_str("\\b"),
            0x0c => string.push_str("\\f"),
            0x0a => string.push_str("\\n"),
            0x0d => string.push_str("\\r"),
            0x09 => string.push_str("\\t"),
            0x20..=0x21 | 0x23..=0x5b | 0x5d..=0x10ffff => string.push(c),
            b => {
                if b < 0x10000 {
                    string.push_str(&format!("\\u{:04x}", b))
                } else {
                    let mut buf: [u16; 2] = [0; 2];
                    let utf16 = c.encode_utf16(&mut buf);

                    for x in utf16 {
                        string.push_str(&format!("\\u{:04x}", x))
                    }
                }
            }
        }
    }

    string.push('"');

    string
}

fn array_to_string(array: &[Value]) -> String {
    let inner = array
        .iter()
        .map(|v| v.serialize())
        .fold(String::new(), |acc, s| acc + &s + ",");

    if inner.ends_with(',') {
        "[".to_string() + &inner[..inner.len() - 1] + "]"
    } else {
        "[".to_string() + &inner + "]"
    }
}

fn object_to_string(object: &[(String, Value)]) -> String {
    let inner = object
        .iter()
        .map(|(k, v)| (Value::String(k.clone()).serialize(), v.serialize()))
        .fold(String::new(), |acc, (k, v)| acc + &k + ":" + &v + ",");

    if inner.ends_with(',') {
        "{".to_string() + &inner[..inner.len() - 1] + "}"
    } else {
        "{".to_string() + &inner + "}"
    }
}

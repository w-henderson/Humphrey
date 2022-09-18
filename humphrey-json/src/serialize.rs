//! Provides JSON serialization functionality.

use crate::Value;

use std::fmt::Write;

impl Value {
    /// Serialize a JSON value into a string.
    pub fn serialize(&self) -> String {
        match self {
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => string_to_string(s),
            Value::Array(a) => array_to_string(a, None),
            Value::Object(o) => object_to_string(o, None),
        }
    }

    /// Serialize a JSON value into a string, with indentation.
    pub fn serialize_pretty(&self, indent: usize) -> String {
        self.serialize_pretty_indent(0, indent)
    }

    fn serialize_pretty_indent(&self, indent: usize, indent_size: usize) -> String {
        match self {
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => string_to_string(s),
            Value::Array(a) => array_to_string(a, Some((indent, indent_size))),
            Value::Object(o) => object_to_string(o, Some((indent, indent_size))),
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
                    write!(string, "\\u{:04x}", b).ok();
                } else {
                    let mut buf: [u16; 2] = [0; 2];
                    let utf16 = c.encode_utf16(&mut buf);

                    for x in utf16 {
                        write!(string, "\\u{:04x}", x).ok();
                    }
                }
            }
        }
    }

    string.push('"');

    string
}

fn array_to_string(array: &[Value], indent: Option<(usize, usize)>) -> String {
    if array.is_empty() {
        return "[]".to_string();
    }

    let mut inner = array
        .iter()
        .map(|v| match indent {
            Some((indent, indent_size)) => {
                v.serialize_pretty_indent(indent + indent_size, indent_size)
            }
            None => v.serialize(),
        })
        .fold(
            {
                let mut s = String::with_capacity(256);
                s.push('[');
                s
            },
            |acc, s| match indent {
                Some((indent, indent_size)) => {
                    acc + "\n" + &" ".repeat(indent + indent_size) + &s + ","
                }
                None => acc + &s + ",",
            },
        );

    if inner.ends_with(',') {
        inner.pop();
    }

    if let Some((indent, _)) = indent {
        inner.push('\n');

        for _ in 0..indent {
            inner.push(' ');
        }
    }

    inner.push(']');

    inner
}

fn object_to_string(object: &[(String, Value)], indent: Option<(usize, usize)>) -> String {
    if object.is_empty() {
        return "{}".to_string();
    }

    let mut inner = object
        .iter()
        .map(|(k, v)| match indent {
            Some((indent, indent_size)) => (
                string_to_string(k),
                v.serialize_pretty_indent(indent + indent_size, indent_size),
            ),
            None => (string_to_string(k), v.serialize()),
        })
        .fold(
            {
                let mut s = String::with_capacity(256);
                s.push('{');
                s
            },
            |acc, (k, v)| match indent {
                Some((indent, indent_size)) => {
                    acc + "\n" + &" ".repeat(indent + indent_size) + &k + ": " + &v + ","
                }
                None => acc + &k + ":" + &v + ",",
            },
        );

    if inner.ends_with(',') {
        inner.pop();
    }

    if let Some((indent, _)) = indent {
        inner.push('\n');

        for _ in 0..indent {
            inner.push(' ');
        }
    }

    inner.push('}');

    inner
}

//! Provides the `Value` struct for interfacing with JSON values.

use std::collections::HashMap;
use std::ops::Index;

/// Represents a JSON value.
///
/// ## Constructing
/// The `true` value, for example, can be constructed as any of the following:
/// ```rs
/// let _ = Value::Bool(true);
/// let _ = json!(true);
/// let _ = Value::parse("true");
/// ```
#[derive(Clone, Debug)]
pub enum Value {
    /// The `null` value.
    Null,
    /// A boolean value, `true` or `false`.
    ///
    /// Can be extracted with `as_bool()`.
    Bool(bool),
    /// A numeric value, stored as a float in accordance with the specification.
    ///
    /// Can be extracted with `as_number()`.
    Number(f64),
    /// A UTF-8 string value.
    ///
    /// Can be extracted with `as_str()`.
    String(String),
    /// An array of values.
    ///
    /// Can be extracted with `as_array()`.
    Array(Vec<Value>),
    /// An object mapping of values.
    ///
    /// Can be extracted with `as_map()`.
    Object(HashMap<String, Value>),
}

impl Value {
    /// Returns the encapsulated boolean value, or `None` if it is not a boolean data type.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Returns the encapsulated numeric value, or `None` if it is not a numeric data type.
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Returns the encapsulated string value, or `None` if it is not a string data type.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Returns the encapsulated array value, or `None` if it not an array data type.
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Returns the encapsulated hash map value, or `None` if it not an object data type.
    pub fn as_map(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Value::Object(o) => Some(o),
            _ => None,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(l0), Self::Bool(r0)) => *l0 == *r0,
            (Self::Number(l0), Self::Number(r0)) => *l0 == *r0,
            (Self::String(l0), Self::String(r0)) => *l0 == *r0,
            (Self::Array(l0), Self::Array(r0)) => *l0 == *r0,
            (Self::Object(l0), Self::Object(r0)) => *l0 == *r0,
            (Self::Null, Self::Null) => true,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl Index<usize> for Value {
    type Output = Value;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Self::Array(a) => &a[index],
            _ => panic!("Indexing a non-array value with an integer index"),
        }
    }
}

impl Index<&str> for Value {
    type Output = Value;

    fn index(&self, index: &str) -> &Self::Output {
        match self {
            Self::Object(o) => &o[index],
            _ => panic!("Indexing a non-object value with a string index"),
        }
    }
}

//! Provides the `Value` struct for interfacing with JSON values.

use crate::indexing::Index;

/// Represents a JSON value.
///
/// ## Constructing
/// The `true` value, for example, can be constructed as any of the following:
/// ```
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
    /// Can be extracted with `as_object()`.
    Object(Vec<(String, Value)>),
}

impl Value {
    /// Gets the value at the given index, or `None` if not found.
    ///
    /// ```
    /// let x = array.get(0);
    /// let y = object.get("name");
    /// ```
    pub fn get<I>(&self, index: I) -> Option<&Value>
    where
        I: Index,
    {
        index.json_index(self)
    }

    /// Gets a mutable reference to the value at the given index, or `None` if not found.
    ///
    /// ```
    /// let x = array.get_mut(0);
    /// let y = object.get_mut("name");
    /// ```
    ///
    /// **Warning:** this creates a new null value at the given index if it does not exist in an object.
    pub fn get_mut<I>(&mut self, index: I) -> Option<&mut Value>
    where
        I: Index,
    {
        index.json_index_mut(self)
    }

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

    /// Returns the encapsulated object value, or `None` if it not an object data type.
    pub fn as_object(&self) -> Option<&Vec<(String, Value)>> {
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

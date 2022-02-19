use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

impl Value {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s.as_str()),
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

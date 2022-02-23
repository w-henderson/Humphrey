//! Provides useful traits for working with JSON values.

use crate::Value;

use std::error::Error;

/// Represents the ability of a type to be converted into a JSON value.
///
/// This trait is implemented for both core string types, the boolean type, all numeric types, and all `Option<T>` where `T` implements the trait.
pub trait IntoJson {
    /// Creates a JSON value from itself.
    fn to_json(&self) -> Value;
}

/// Represents the ability of a type to be constructed from a JSON value.
pub trait FromJson {
    /// Constructs itself from a JSON value.
    fn from_json(value: &Value) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
}

impl<T> From<T> for Value
where
    T: IntoJson,
{
    fn from(value: T) -> Self {
        value.to_json()
    }
}

impl IntoJson for bool {
    fn to_json(&self) -> Value {
        Value::Bool(*self)
    }
}

impl IntoJson for String {
    fn to_json(&self) -> Value {
        Value::String(self.to_owned())
    }
}

impl IntoJson for &str {
    fn to_json(&self) -> Value {
        Value::String(self.to_string())
    }
}

impl<T> IntoJson for Option<T>
where
    T: IntoJson,
{
    fn to_json(&self) -> Value {
        match self {
            Some(v) => v.to_json(),
            None => Value::Null,
        }
    }
}

macro_rules! impl_into_json_for_number {
    ($($t:ty),*) => {
        $(
            impl IntoJson for $t {
                fn to_json(&self) -> Value {
                    Value::Number(*self as f64)
                }
            }
        )*
    };
}

impl_into_json_for_number!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64
);

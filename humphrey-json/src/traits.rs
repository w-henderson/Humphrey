use crate::Value;

use std::error::Error;

pub trait IntoJson {
    fn to_json(&self) -> Value;
}

pub trait FromJson {
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

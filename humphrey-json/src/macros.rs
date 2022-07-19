//! Provides the `json!` macro for creating JSON values and the `json_map!` macro
//!   for serializing/deserializing them to and from Rust data structures.

/// Create a JSON value from JSON-like syntax.
///
/// ## Usage
/// The macro allows you to create any kind of JSON value.
///
/// ```
/// let value = json!({
///     "key": "value",
///     "array": [1, 2, 3],
///     "empty": null,
///     "bool": true
/// });
/// ```
///
/// It can also be used as a shorthand to create literal values.
///
/// ```
/// assert_eq!(json!(true), Value::Bool(true));
/// assert_eq!(json!(1234), Value::Number(1234.0));
/// assert_eq!(json!("Hello, world!"), Value::String("Hello, world!".into()));
/// ```
#[macro_export]
macro_rules! json {
    () => {
        $crate::Value::Null
    };
    (null) => {
        $crate::Value::Null
    };
    ([ $( $el:tt ),* ]) => {
        $crate::Value::Array(vec![ $( json!($el) ),* ])
    };
    ({}) => {
        $crate::Value::Object(Vec::new())
    };
    ({ $( $k:tt : $v:tt ),* }) => {
        $crate::Value::Object(vec![
            $( ($k.to_string(), json!($v)) ),*
        ])
    };
    ({ $( $k:tt : $v:tt, )* }) => {
        $crate::Value::Object(vec![
            $( ($k.to_string(), json!($v)) ),*
        ])
    };
    ($v:tt) => {
        $crate::Value::from($v)
    };
}

/// Specifies a mapping between a Rust data structure and a JSON value.
///
/// Behind the scenes, this macro automatically generates a `FromJson` and an `IntoJson` implementation.
/// It is functionally equivalent to Serde's `Serialize` and `Deserialize` derive macros.
///
/// ## Usage
/// The first argument in a comma-separated list is the type of the Rust data structure.
/// The rest of the arguments specify the mapping from the type to its JSON representation.
///
/// ```
/// use humphrey_json::prelude::*;
///
/// #[derive(Debug)]
/// struct Point {
///     x: i32,
///     y: i32,
/// }
///
/// json_map! {
///     Point,
///     x => "x",
///     y => "y"
/// }
///
/// fn main() {
///     let point = Point { x: 1, y: 2 };
///     let serialized = humphrey_json::to_string(&point);
///
///     println!("serialized = {}", serialized);
///     let deserialized: Point = humphrey_json::from_str(serialized).unwrap();
///
///     println!("deserialized = {:?}", deserialized);
/// }
/// ```
#[macro_export]
macro_rules! json_map {
    ($t:ty, $($struct_field:tt => $json_field:expr),*) => {
        impl $crate::traits::FromJson for $t {
            fn from_json(value: &$crate::Value) -> Result<Self, $crate::error::ParseError> {
                Ok(Self {
                    $($struct_field: $crate::traits::FromJson::from_json(value.get($json_field).unwrap_or(&$crate::Value::Null))?),*
                })
            }
        }

        impl $crate::traits::IntoJson for $t {
            fn to_json(&self) -> $crate::Value {
                json!({
                    $($json_field: (&self.$struct_field)),*
                })
            }
        }
    };
}

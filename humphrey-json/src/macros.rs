//! Provides the `json!` macro for creating JSON values and the `json_map!` macro
//!   for serializing/deserializing them to and from Rust data structures.

// This module is highly inspired by the `serde_json` crate's macro implementation.
// However, since we use a `Vec` instead of a `Map` for storing JSON objects, the implementation is slightly simpler.
//
// Reference:
// - [serde_json::macros](https://github.com/serde-rs/json/blob/94019a31c6036dc4ebb9afc44a214f950caf0d1f/src/macros.rs)

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
    ([ $($elems:tt)* ]) => {
        $crate::Value::Array(json_array_internal!([] $($elems)*))
    };
    ({}) => {
        $crate::Value::Object(Vec::new())
    };
    ({ $($elems:tt)* }) => {
        $crate::Value::Object(json_object_internal!([] $($elems)*))
    };
    ($v:expr) => {
        $crate::Value::from($v)
    };
}

/// Internal macro, do not use.
#[macro_export]
#[doc(hidden)]
macro_rules! json_array_internal {
    ([ $($elems:expr,)* ]) => {
        vec![ $( $elems, )* ]
    };

    ([ $($elems:expr),* ]) => {
        vec![ $( $elems ),* ]
    };

    // Next value is `null`.
    ([ $($elems:expr,)* ] null $($rest:tt)*) => {
        json_array_internal!([ $($elems,)* $crate::Value::Null, ])
    };

    // Next value is an array.
    ([ $($elems:expr,)* ] [ $($array:tt)* ] $($rest:tt)*) => {
        json_array_internal!([ $($elems,)* json!([ $($array)* ]), ] $($rest)*)
    };

    // Next value is an object.
    ([ $($elems:expr,)* ] { $($object:tt)* } $($rest:tt)*) => {
        json_array_internal!([ $($elems,)* json!({ $($object)* }), ] $($rest)*)
    };

    // Next value is an expression.
    ([ $($elems:expr,)* ] $value:expr, $($rest:tt)*) => {
        json_array_internal!([ $($elems,)* json!($value), ] $($rest)*)
    };

    // Last value is an expression.
    ([ $($elems:expr,)* ] $value:expr) => {
        json_array_internal!([ $($elems,)* json!($value) ])
    };

    // Comma.
    ([ $($elems:expr,)* ] , $($rest:tt)*) => {
        json_array_internal!([ $($elems,)* ] $($rest)*)
    };
}

/// Internal macro, do not use.
#[macro_export]
#[doc(hidden)]
macro_rules! json_object_internal {
    ([ $($elems:expr,)* ]) => {
        vec![ $( $elems, )* ]
    };

    ([ $($elems:expr),* ]) => {
        vec![ $( $elems ),* ]
    };

    // Next value is `null`.
    ([ $($elems:expr,)* ] $key:tt : null $($rest:tt)*) => {
        $crate::json_object_internal!([ $($elems,)* ($key.to_string(), $crate::Value::Null), ] $($rest)*)
    };

    // Next value is an array.
    ([ $($elems:expr,)* ] $key:tt : [ $($array:tt)* ] $($rest:tt)*) => {
        $crate::json_object_internal!([ $($elems,)* ($key.to_string(), $crate::json!([ $($array)* ])), ] $($rest)*)
    };

    // Next value is an object.
    ([ $($elems:expr,)* ] $key:tt : { $($object:tt)* } $($rest:tt)*) => {
        $crate::json_object_internal!([ $($elems,)* ($key.to_string(), $crate::json!({ $($object)* })), ] $($rest)*)
    };

    // Next value is an expression.
    ([ $($elems:expr,)* ] $key:tt : $value:expr, $($rest:tt)*) => {
        $crate::json_object_internal!([ $($elems,)* ($key.to_string(), $crate::json!($value)), ] $($rest)*)
    };

    // Last value is an expression.
    ([ $($elems:expr,)* ] $key:tt : $value:expr) => {
        $crate::json_object_internal!([ $($elems,)* ($key.to_string(), $crate::json!($value)), ])
    };

    // Comma.
    ([ $($elems:expr,)* ] , $($rest:tt)*) => {
        $crate::json_object_internal!([ $($elems,)* ] $($rest)*)
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
                $crate::json!({
                    $($json_field: (&self.$struct_field)),*
                })
            }
        }
    };
}

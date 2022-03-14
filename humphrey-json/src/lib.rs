//! Humphrey JSON is a library facilitating the serialization and deserialization of JSON data. It is designed for web applications, but can be used in other contexts, and is well-integrated with the Humphrey web server.
//!
//! Learn more about Humphrey JSON [here](https://humphrey.whenderson.dev/json/index.html).

#![warn(missing_docs)]
#![allow(clippy::needless_doctest_main)]

pub mod error;
pub mod indexing;
pub mod parser;
pub mod serialize;
pub mod traits;
pub mod value;

#[macro_use]
pub mod macros;

#[cfg(test)]
mod tests;

/// Brings useful traits and macros into scope.
///
/// ```
/// use humphrey_json::prelude::*;
/// ```
pub mod prelude {
    pub use crate::json;
    pub use crate::json_map;
    pub use crate::traits::*;
}

pub use value::Value;

/// Deserialize a JSON string into a Rust data structure.
///
/// ## Usage
/// ```
/// use humphrey_json::prelude::*;
///
/// #[derive(Debug)]
/// struct User {
///     name: String,
///     country: String,
/// }
///
/// json_map! {
///     User,
///     name => "name",
///     country => "country"
/// }
///
/// fn main() {
///     let json_string = r#"
///         {
///             "name": "William Henderson",
///             "country": "United Kingdom"
///         }"#;
///
///     let user: User = humphrey_json::from_str(json_string).unwrap();
///
///     println!("{:?}", user);
/// }
/// ```
///
/// ## Errors
/// This function returns a `ParseError` if the JSON string is invalid,
///   or if the JSON string is missing a required field.
pub fn from_str<T, S>(s: S) -> Result<T, error::ParseError>
where
    T: traits::FromJson,
    S: AsRef<str>,
{
    Value::parse(s)
        .map_err(|e| e.into())
        .and_then(|v| T::from_json(&v))
}

/// Serialize a Rust data structure into a JSON string.
///
/// ## Usage
/// ```
/// use humphrey_json::prelude::*;
///
/// struct User {
///     name: String,
///     country: String,
/// }
///
/// json_map! {
///     User,
///     name => "name",
///     country => "country"
/// }
///
/// fn main() {
///     let user = User {
///         name: "William Henderson".into(),
///         country: "United Kingdom".into(),
///     };
///
///     let json_string = humphrey_json::to_string(&user);
///
///     println!("{}", json_string);
/// }
/// ```
pub fn to_string<T>(v: &T) -> String
where
    T: traits::IntoJson,
{
    v.to_json().serialize()
}

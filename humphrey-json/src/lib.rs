//! Humphrey JSON is a library facilitating the serialization and deserialization of JSON data. It is designed for web applications, but can be used in other contexts, and is well-integrated with the Humphrey web server.
//!
//! Learn more about Humphrey JSON [here (coming soon)](https://humphrey.whenderson.dev/json/index.html).

#![warn(missing_docs)]

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
/// ```rs
/// use humphrey_json::prelude::*;
/// ```
pub mod prelude {
    pub use crate::json;
    pub use crate::json_map;
    pub use crate::traits::*;
}

pub use value::Value;

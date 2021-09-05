use super::request::RequestError;

use std::fmt::Display;

/// Represents an HTTP method.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
}

impl Method {
    /// Attempts to convert from the HTTP verb into an enum variant.
    ///
    /// ## Example
    /// ```
    /// let method = Method::from_name("GET");
    /// assert_eq!(method, Method::Get);
    /// ```
    pub fn from_name(name: &str) -> Result<Self, RequestError> {
        match name {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            _ => Err(RequestError::Request),
        }
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Method::Get => "GET",
                Method::Post => "POST",
                Method::Put => "PUT",
                Method::Delete => "DELETE",
            }
        )
    }
}

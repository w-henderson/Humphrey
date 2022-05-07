//! Contains the CORS implementation for Humphrey.

use crate::http::headers::{HeaderLike, HeaderType, Headers};
use crate::http::method::Method;

#[derive(Clone)]
enum Wildcardable<T> {
    Wildcard,
    Value(T),
}

/// Represents a CORS configuration.
///
/// Cross-origin resource sharing (CORS) is a mechanism that allows a server to indicate any origins other than
///   its own from which a browser should permit loading resources.
///
/// Learn more about CORS at the [MDN docs](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS).
#[derive(Clone, Default)]
pub struct Cors {
    allowed_origins: Wildcardable<Vec<String>>,
    allowed_methods: Wildcardable<Vec<Method>>,
    allowed_headers: Wildcardable<Vec<String>>,
}

impl Cors {
    /// Creates a new CORS configuration with no allowed origins.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new CORS configuration with every origin, header and method allowed.
    ///
    /// This corresponds to the following headers:
    /// ```text
    /// Access-Control-Allow-Origin: *
    /// Access-Control-Allow-Headers: *
    /// Access-Control-Allow-Methods: * (although this is implied)
    /// ```
    pub fn wildcard() -> Self {
        Self {
            allowed_origins: Wildcardable::Wildcard,
            allowed_methods: Wildcardable::Wildcard,
            allowed_headers: Wildcardable::Wildcard,
        }
    }

    /// Sets the allowed origins to "*", allowing all origins.
    pub fn with_wildcard_origin(mut self) -> Self {
        self.allowed_origins = Wildcardable::Wildcard;
        self
    }

    /// Sets the allowed methods to "*", allowing all methods.
    pub fn with_wildcard_methods(mut self) -> Self {
        self.allowed_methods = Wildcardable::Wildcard;
        self
    }

    /// Sets the allowed headers to "*", allowing all headers.
    pub fn with_wildcard_headers(mut self) -> Self {
        self.allowed_headers = Wildcardable::Wildcard;
        self
    }

    /// Adds the allowed origin.
    pub fn with_origin(mut self, origin: &str) -> Self {
        match self.allowed_origins {
            Wildcardable::Wildcard => (),
            Wildcardable::Value(ref mut origins) => origins.push(origin.to_string()),
        }
        self
    }

    /// Adds the allowed method.
    pub fn with_method(mut self, method: Method) -> Self {
        match self.allowed_methods {
            Wildcardable::Wildcard => (),
            Wildcardable::Value(ref mut methods) => methods.push(method),
        }
        self
    }

    /// Adds the allowed header.
    pub fn with_header(mut self, header: impl HeaderLike) -> Self {
        match self.allowed_headers {
            Wildcardable::Wildcard => (),
            Wildcardable::Value(ref mut headers) => headers.push(header.to_header().to_string()),
        }
        self
    }

    /// Sets the appropriate headers for the CORS configuration.
    pub(crate) fn set_headers(&self, headers: &mut Headers) {
        if headers.get(HeaderType::AccessControlAllowOrigin).is_none() {
            match self.allowed_origins {
                Wildcardable::Wildcard => {
                    headers.add(HeaderType::AccessControlAllowOrigin, "*");
                }
                Wildcardable::Value(ref origins) if !origins.is_empty() => {
                    headers.add(HeaderType::AccessControlAllowOrigin, &origins.join(", "));
                }
                _ => (),
            }
        }

        if headers.get(HeaderType::AccessControlAllowMethods).is_none() {
            match self.allowed_methods {
                Wildcardable::Value(ref methods) if !methods.is_empty() => {
                    headers.add(
                        HeaderType::AccessControlAllowMethods,
                        &methods
                            .iter()
                            .map(|m| m.to_string())
                            .collect::<Vec<_>>()
                            .join(", "),
                    );
                }
                _ => (),
            }
        }

        if headers.get(HeaderType::AccessControlAllowHeaders).is_none() {
            match self.allowed_headers {
                Wildcardable::Wildcard => {
                    headers.add(HeaderType::AccessControlAllowHeaders, "*");
                }
                Wildcardable::Value(ref allowed_headers) if !allowed_headers.is_empty() => {
                    headers.add(
                        HeaderType::AccessControlAllowHeaders,
                        &allowed_headers.join(", "),
                    );
                }
                _ => (),
            }
        }
    }
}

impl<T> Default for Wildcardable<T>
where
    T: Default,
{
    fn default() -> Self {
        Wildcardable::Value(T::default())
    }
}

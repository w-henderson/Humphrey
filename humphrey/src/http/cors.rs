//! Contains the CORS implementation for Humphrey.

use crate::http::headers::{HeaderLike, HeaderType, Headers};
use crate::http::method::Method;

#[derive(Clone)]
enum Wildcardable<T> {
    Wildcard,
    Value(T),
}

#[derive(Clone, Default)]
pub struct Cors {
    allowed_origins: Wildcardable<Vec<String>>,
    allowed_methods: Wildcardable<Vec<Method>>,
    allowed_headers: Wildcardable<Vec<String>>,
}

impl Cors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn wildcard() -> Self {
        Self {
            allowed_origins: Wildcardable::Wildcard,
            allowed_methods: Wildcardable::Wildcard,
            allowed_headers: Wildcardable::Wildcard,
        }
    }

    pub fn with_wildcard_origin(mut self) -> Self {
        self.allowed_origins = Wildcardable::Wildcard;
        self
    }

    pub fn with_wildcard_methods(mut self) -> Self {
        self.allowed_methods = Wildcardable::Wildcard;
        self
    }

    pub fn with_wildcard_headers(mut self) -> Self {
        self.allowed_headers = Wildcardable::Wildcard;
        self
    }

    pub fn with_origin(mut self, origin: &str) -> Self {
        match self.allowed_origins {
            Wildcardable::Wildcard => (),
            Wildcardable::Value(ref mut origins) => origins.push(origin.to_string()),
        }
        self
    }

    pub fn with_method(mut self, method: Method) -> Self {
        match self.allowed_methods {
            Wildcardable::Wildcard => (),
            Wildcardable::Value(ref mut methods) => methods.push(method),
        }
        self
    }

    pub fn with_header(mut self, header: impl HeaderLike) -> Self {
        match self.allowed_headers {
            Wildcardable::Wildcard => (),
            Wildcardable::Value(ref mut headers) => headers.push(header.to_header().to_string()),
        }
        self
    }

    pub(crate) fn set_headers(&self, headers: &mut Headers) {
        match self.allowed_origins {
            Wildcardable::Wildcard => {
                headers.add(HeaderType::AccessControlAllowOrigin, "*");
            }
            Wildcardable::Value(ref origins) if !origins.is_empty() => {
                headers.add(HeaderType::AccessControlAllowOrigin, &origins.join(", "));
            }
            _ => (),
        }

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

impl<T> Default for Wildcardable<T>
where
    T: Default,
{
    fn default() -> Self {
        Wildcardable::Value(T::default())
    }
}

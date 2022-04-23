//! Provides a basic cookie implementation according to [RFC 6265](http://tools.ietf.org/html/rfc6265).

use crate::http::headers::Header;

use std::time::Duration;

/// Represents an HTTP cookie as in the `Cookie` header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cookie {
    /// The name of the cookie.
    pub name: String,
    /// The value of the cookie.
    pub value: String,
}

/// Represents an HTTP cookie as in the `Set-Cookie` header.
///
/// Contains additional information about the cookie, such as its expiration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SetCookie {
    /// The name of the cookie.
    pub name: String,
    /// The value of the cookie.
    pub value: String,
    /// The expiry date of the cookie as an HTTP timestamp.
    pub expires: Option<String>,
    /// The maximum age of the cookie.
    pub max_age: Option<Duration>,
    /// The domain of the cookie.
    pub domain: Option<String>,
    /// The path of the cookie.
    pub path: Option<String>,
    /// Whether the cookie is secure.
    pub secure: bool,
    /// Whether the cookie is HTTP-only.
    pub http_only: bool,
    /// The SameSite configuration of the cookie.
    pub same_site: Option<SameSite>,
}

/// Represents the SameSite value of the cookie.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SameSite {
    /// Cookies will only be sent in a first-party context and not be sent along with requests
    ///   initiated by third party websites.
    Strict,
    /// Cookies are not sent on normal cross-site subrequests (for example to load images or frames
    ///   into a third party site), but are sent when a user is navigating to the origin site
    ///   (i.e., when following a link).
    Lax,
    /// Cookies will be sent in all contexts, i.e. in responses to both first-party and cross-origin requests.
    /// If SameSite=None is set, the cookie Secure attribute must also be set (or the cookie will be blocked).
    None,
}

impl Cookie {
    /// Create a new cookie with the given name and value.
    pub fn new(name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        Self {
            name: name.as_ref().to_string(),
            value: value.as_ref().to_string(),
        }
    }

    /// Convert a collection of cookies into a `Cookie` header.
    pub fn to_header(cookies: impl AsRef<[Cookie]>) -> Option<Header> {
        let cookies = cookies.as_ref();

        if cookies.is_empty() {
            return None;
        }

        let mut value = String::with_capacity(cookies.len() * 32);

        for cookie in cookies {
            value.push_str(&cookie.name);
            value.push('=');
            value.push_str(&cookie.value);
            value.push_str("; ");
        }

        Some(Header::new("Cookie", &value[..value.len() - 2]))
    }
}

impl SetCookie {
    /// Create a new cookie with the given name and value.
    pub fn new(name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        Self {
            name: name.as_ref().to_string(),
            value: value.as_ref().to_string(),
            expires: None,
            max_age: None,
            domain: None,
            path: None,
            secure: false,
            http_only: false,
            same_site: None,
        }
    }

    /// Set the expiry date of the cookie.
    ///
    /// **Warning:** This must be a valid HTTP timestamp.
    pub fn with_expires(mut self, expires: impl AsRef<str>) -> Self {
        self.expires = Some(expires.as_ref().to_string());
        self
    }

    /// Set the maximum age of the cookie.
    pub fn with_max_age(mut self, max_age: Duration) -> Self {
        self.max_age = Some(max_age);
        self
    }

    /// Set the domain of the cookie.
    pub fn with_domain(mut self, domain: impl AsRef<str>) -> Self {
        self.domain = Some(domain.as_ref().to_string());
        self
    }

    /// Set the path of the cookie.
    pub fn with_path(mut self, path: impl AsRef<str>) -> Self {
        self.path = Some(path.as_ref().to_string());
        self
    }

    /// Set the secure flag of the cookie.
    pub fn with_secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    /// Set the HTTP-only flag of the cookie.
    pub fn with_http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;
        self
    }

    /// Set the SameSite configuration of the cookie.
    pub fn with_same_site(mut self, same_site: SameSite) -> Self {
        self.same_site = Some(same_site);
        self
    }
}

impl From<SetCookie> for Header {
    fn from(cookie: SetCookie) -> Self {
        let mut value = format!("{}={}", cookie.name, cookie.value);

        if let Some(expires) = cookie.expires {
            value = format!("{}; Expires={}", value, expires);
        }

        if let Some(max_age) = cookie.max_age {
            value = format!("{}; Max-Age={}", value, max_age.as_secs());
        }

        if let Some(domain) = cookie.domain {
            value = format!("{}; Domain={}", value, domain);
        }

        if let Some(path) = cookie.path {
            value = format!("{}; Path={}", value, path);
        }

        if let Some(same_site) = cookie.same_site {
            value = format!(
                "{}; SameSite={}",
                value,
                match same_site {
                    SameSite::Strict => "Strict",
                    SameSite::Lax => "Lax",
                    SameSite::None => "None",
                }
            );
        }

        if cookie.secure {
            value = format!("{}; Secure", value);
        }

        if cookie.http_only {
            value = format!("{}; HttpOnly", value);
        }

        Header::new("Set-Cookie", value)
    }
}

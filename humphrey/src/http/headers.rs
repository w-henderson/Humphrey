use std::collections::BTreeMap;

/// Alias for a map of request headers and their values.
pub type RequestHeaderMap = BTreeMap<RequestHeader, String>;

/// Alias for a map of response headers and their values.
pub type ResponseHeaderMap = BTreeMap<ResponseHeader, String>;

/// Represents a header received in a request.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum RequestHeader {
    Accept,
    AcceptCharset,
    AcceptEncoding,
    AcceptLanguage,
    AccessControlRequestMethod,
    Authorization,
    CacheControl,
    Connection,
    ContentEncoding,
    ContentLength,
    ContentType,
    Cookie,
    Date,
    Expect,
    Forwarded,
    From,
    Host,
    Origin,
    Pragma,
    Referer,
    Upgrade,
    UserAgent,
    Via,
    Warning,

    /// Custom header with a lowercase name
    Custom {
        name: String,
    },
}

/// Represents a header sent in a response.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ResponseHeader {
    AccessControlAllowOrigin,
    Age,
    Allow,
    CacheControl,
    Connection,
    ContentDisposition,
    ContentEncoding,
    ContentLanguage,
    ContentLength,
    ContentLocation,
    ContentType,
    Date,
    ETag,
    Expires,
    LastModified,
    Link,
    Location,
    Pragma,
    Server,
    SetCookie,
    Via,
    Warning,

    Custom { name: String },
}

impl PartialOrd for ResponseHeader {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.category() != other.category() {
            self.category().partial_cmp(&other.category())
        } else {
            self.to_string().partial_cmp(&other.to_string())
        }
    }
}

impl Ord for ResponseHeader {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl From<&str> for RequestHeader {
    fn from(name: &str) -> Self {
        match name.to_ascii_lowercase().as_str() {
            "accept" => Self::Accept,
            "accept-charset" => Self::AcceptCharset,
            "accept-encoding" => Self::AcceptEncoding,
            "accept-language" => Self::AcceptLanguage,
            "access-control-request-method" => Self::AccessControlRequestMethod,
            "authorization" => Self::Authorization,
            "cache-control" => Self::CacheControl,
            "connection" => Self::Connection,
            "content-encoding" => Self::ContentEncoding,
            "content-length" => Self::ContentLength,
            "content-type" => Self::ContentType,
            "cookie" => Self::Cookie,
            "date" => Self::Date,
            "expect" => Self::Expect,
            "forwarded" => Self::Forwarded,
            "from" => Self::From,
            "host" => Self::Host,
            "origin" => Self::Origin,
            "pragma" => Self::Pragma,
            "referer" => Self::Referer,
            "upgrade" => Self::Upgrade,
            "user-agent" => Self::UserAgent,
            "via" => Self::Via,
            "warning" => Self::Warning,
            custom => Self::Custom {
                name: custom.to_string(),
            },
        }
    }
}

impl From<&str> for ResponseHeader {
    fn from(name: &str) -> Self {
        match name {
            "Access-Control-Allow-Origin" => Self::AccessControlAllowOrigin,
            "Age" => Self::Age,
            "Allow" => Self::Allow,
            "Cache-Control" => Self::CacheControl,
            "Connection" => Self::Connection,
            "Content-Disposition" => Self::ContentDisposition,
            "Content-Encoding" => Self::ContentEncoding,
            "Content-Language" => Self::ContentLanguage,
            "Content-Length" => Self::ContentLength,
            "Content-Location" => Self::ContentLocation,
            "Content-Type" => Self::ContentType,
            "Date" => Self::Date,
            "ETag" => Self::ETag,
            "Expires" => Self::Expires,
            "Last-Modified" => Self::LastModified,
            "Link" => Self::Link,
            "Location" => Self::Location,
            "Pragma" => Self::Pragma,
            "Server" => Self::Server,
            "Set-Cookie" => Self::SetCookie,
            "Via" => Self::Via,
            "Warning" => Self::Warning,
            custom => Self::Custom {
                name: custom.to_string(),
            },
        }
    }
}

impl ToString for RequestHeader {
    fn to_string(&self) -> String {
        if let RequestHeader::Custom { name } = self {
            return name.clone();
        }

        match self {
            RequestHeader::Accept => "Accept",
            RequestHeader::AcceptCharset => "Accept-Charset",
            RequestHeader::AcceptEncoding => "Accept-Encoding",
            RequestHeader::AcceptLanguage => "Accept-Language",
            RequestHeader::AccessControlRequestMethod => "Access-Control-Request-Method",
            RequestHeader::Authorization => "Authorization",
            RequestHeader::CacheControl => "Cache-Control",
            RequestHeader::Connection => "Connection",
            RequestHeader::ContentEncoding => "Content-Encoding",
            RequestHeader::ContentLength => "Content-Length",
            RequestHeader::ContentType => "Content-Type",
            RequestHeader::Cookie => "Cookie",
            RequestHeader::Date => "Date",
            RequestHeader::Expect => "Expect",
            RequestHeader::Forwarded => "Forwarded",
            RequestHeader::From => "From",
            RequestHeader::Host => "Host",
            RequestHeader::Origin => "Origin",
            RequestHeader::Pragma => "Pragma",
            RequestHeader::Referer => "Referer",
            RequestHeader::Upgrade => "Upgrade",
            RequestHeader::UserAgent => "User-Agent",
            RequestHeader::Via => "Via",
            RequestHeader::Warning => "Warning",
            _ => "",
        }
        .to_string()
    }
}

impl ToString for ResponseHeader {
    fn to_string(&self) -> String {
        if let ResponseHeader::Custom { name } = self {
            return name.clone();
        }

        match self {
            ResponseHeader::AccessControlAllowOrigin => "Access-Control-Allow-Origin",
            ResponseHeader::Age => "Age",
            ResponseHeader::Allow => "Allow",
            ResponseHeader::CacheControl => "Cache-Control",
            ResponseHeader::Connection => "Connection",
            ResponseHeader::ContentDisposition => "Content-Disposition",
            ResponseHeader::ContentEncoding => "Content-Encoding",
            ResponseHeader::ContentLanguage => "Content-Language",
            ResponseHeader::ContentLength => "Content-Length",
            ResponseHeader::ContentLocation => "Content-Location",
            ResponseHeader::ContentType => "Content-Type",
            ResponseHeader::Date => "Date",
            ResponseHeader::ETag => "ETag",
            ResponseHeader::Expires => "Expires",
            ResponseHeader::LastModified => "Last-Modified",
            ResponseHeader::Link => "Link",
            ResponseHeader::Location => "Location",
            ResponseHeader::Pragma => "Pragma",
            ResponseHeader::Server => "Server",
            ResponseHeader::SetCookie => "Set-Cookie",
            ResponseHeader::Via => "Via",
            ResponseHeader::Warning => "Warning",
            _ => "",
        }
        .to_string()
    }
}

/// Represents a category of headers, as defined in [RFC 2616, section 4.2](https://datatracker.ietf.org/doc/html/rfc2616#section-4.2).
/// Used for ordering headers in responses.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum HeaderCategory {
    General,
    Response,
    Entity,
    Other,
}

impl ResponseHeader {
    fn category(&self) -> HeaderCategory {
        match self {
            ResponseHeader::AccessControlAllowOrigin => HeaderCategory::Other,
            ResponseHeader::Age => HeaderCategory::Response,
            ResponseHeader::Allow => HeaderCategory::Entity,
            ResponseHeader::CacheControl => HeaderCategory::General,
            ResponseHeader::Connection => HeaderCategory::General,
            ResponseHeader::ContentDisposition => HeaderCategory::Entity,
            ResponseHeader::ContentEncoding => HeaderCategory::Entity,
            ResponseHeader::ContentLanguage => HeaderCategory::Entity,
            ResponseHeader::ContentLength => HeaderCategory::Entity,
            ResponseHeader::ContentLocation => HeaderCategory::Entity,
            ResponseHeader::ContentType => HeaderCategory::Entity,
            ResponseHeader::Date => HeaderCategory::General,
            ResponseHeader::ETag => HeaderCategory::Response,
            ResponseHeader::Expires => HeaderCategory::Entity,
            ResponseHeader::LastModified => HeaderCategory::Entity,
            ResponseHeader::Link => HeaderCategory::Other,
            ResponseHeader::Location => HeaderCategory::Response,
            ResponseHeader::Pragma => HeaderCategory::General,
            ResponseHeader::Server => HeaderCategory::Response,
            ResponseHeader::SetCookie => HeaderCategory::Other,
            ResponseHeader::Via => HeaderCategory::General,
            ResponseHeader::Warning => HeaderCategory::General,
            ResponseHeader::Custom { name: _ } => HeaderCategory::Other,
        }
    }
}

//! Provides functionality for handling HTTP headers.

use std::collections::BTreeMap;

/// Alias for a map of request headers and their values.
pub type RequestHeaderMap = BTreeMap<RequestHeader, String>;

/// Alias for a map of response headers and their values.
pub type ResponseHeaderMap = BTreeMap<ResponseHeader, String>;

/// Represents a header received in a request.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum RequestHeader {
    /// Informs the server about the types of data that can be sent back.
    Accept,
    /// Informs the server about the accepted character encodings.
    AcceptCharset,
    /// Indicates the content encoding(s) understood by the client, usually compression algorithms.
    AcceptEncoding,
    /// Informs the server about the client's language(s).
    AcceptLanguage,
    /// Indicates the method that will be used for the actual request when performing an OPTIONS request.
    AccessControlRequestMethod,
    /// Provides credentials for HTTP authentication.
    Authorization,
    /// Indicates how the cache should behave.
    CacheControl,
    /// Indicates what should happen to the connection after the request is served.
    Connection,
    /// Lists any encodings used on the payload.
    ContentEncoding,
    /// Indicates the length of the payload body.
    ContentLength,
    /// Indicates the MIME type of the payload body.
    ContentType,
    /// Shares any applicable HTTP cookies with the server.
    Cookie,
    /// Indicates the date and time at which the request was sent.
    Date,
    /// Indicates any expectations that must be met by the server in order to properly serve the request.
    Expect,
    /// May contain reverse proxy information, generally not used in favour of the `X-Forwarded-For` header.
    Forwarded,
    /// Indicates the email address of the client, often used by crawlers.
    From,
    /// Specifies the host to which the request is being sent, e.g. "www.example.com".
    Host,
    /// Indicates the origin that caused the request.
    Origin,
    /// Contains backwards-compatible caching information.
    Pragma,
    /// Indicates the absolute or partial address of the page making the request.
    Referer,
    /// Indicates that the connection is to be upgraded to a different protocol, e.g. WebSocket.
    Upgrade,
    /// Informs the server of basic browser and device information.
    UserAgent,
    /// Contains the addresses of proxies through which the request has been forwarded.
    Via,
    /// Contains information about possible problems with the request.
    Warning,

    /// Custom header with a lowercase name
    Custom {
        /// The name of the header.
        name: String,
    },
}

/// Represents a header sent in a response.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ResponseHeader {
    /// Indicates whether the response can be shared with other origins.
    AccessControlAllowOrigin,
    /// Contains the time in seconds that the object has been cached.
    Age,
    /// The set of methods supported by the resource.
    Allow,
    /// Indicates how the cache should behave.
    CacheControl,
    /// Indicates what should happen to the connection after the request is served.
    Connection,
    /// Indicates whether the response is to be displayed as a webpage or downloaded directly.
    ContentDisposition,
    /// Lists any encodings used on the payload.
    ContentEncoding,
    /// Informs the client of the language of the payload body.
    ContentLanguage,
    /// Indicates the length of the payload body.
    ContentLength,
    /// Indicates an alternative location for the returned data.
    ContentLocation,
    /// Indicates the MIME type of the payload body.
    ContentType,
    /// Indicates the date and time at which the response was created.
    Date,
    /// Identifies a specific version of a resource.
    ETag,
    /// Contains the date and time at which the response is considered expired.
    Expires,
    /// Indicates the date and time at which the response was last modified.
    LastModified,
    /// Provides a means for serialising links in the headers, equivalent to the HTML `<link>` element.
    Link,
    /// Indicates the location at which the resource can be found, used for redirects.
    Location,
    /// Contains backwards-compatible caching information.
    Pragma,
    /// Contains information about the server which served the request.
    Server,
    /// Indicates that the client should set the specified cookies.
    SetCookie,
    /// Indicates the encoding used in the transfer of the payload body.
    TransferEncoding,
    /// Indicates that the connection is to be upgraded to a different protocol, e.g. WebSocket.
    Upgrade,
    /// Contains the addresses of proxies through which the response has been forwarded.
    Via,
    /// Contains information about possible problems with the response.
    Warning,

    /// Custom header with a lowercase name
    Custom {
        /// The name of the header.
        name: String,
    },
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
            "Transfer-Encoding" => Self::TransferEncoding,
            "Upgrade" => Self::Upgrade,
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
            ResponseHeader::TransferEncoding => "Transfer-Encoding",
            ResponseHeader::Upgrade => "Upgrade",
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
            ResponseHeader::TransferEncoding => HeaderCategory::Entity,
            ResponseHeader::Upgrade => HeaderCategory::General,
            ResponseHeader::Via => HeaderCategory::General,
            ResponseHeader::Warning => HeaderCategory::General,
            ResponseHeader::Custom { name: _ } => HeaderCategory::Other,
        }
    }
}

use std::collections::HashMap;

pub trait Header {
    fn default(&self) -> Option<&str>;
}

pub type RequestHeaderMap = HashMap<RequestHeader, String>;
pub type ResponseHeaderMap = HashMap<ResponseHeader, String>;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
    UserAgent,
    Via,
    Warning,

    Custom { name: String },
}

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

impl From<&str> for RequestHeader {
    fn from(name: &str) -> Self {
        match name {
            "Accept" => Self::Accept,
            "Accept-Charset" => Self::AcceptCharset,
            "Accept-Encoding" => Self::AcceptEncoding,
            "Accept-Language" => Self::AcceptLanguage,
            "Access-Control-Request-Method" => Self::AccessControlRequestMethod,
            "Authorization" => Self::Authorization,
            "Cache-Control" => Self::CacheControl,
            "Connection" => Self::Connection,
            "Content-Encoding" => Self::ContentEncoding,
            "Content-Length" => Self::ContentLength,
            "Content-Type" => Self::ContentType,
            "Cookie" => Self::Cookie,
            "Date" => Self::Date,
            "Expect" => Self::Expect,
            "Forwarded" => Self::Forwarded,
            "From" => Self::From,
            "Host" => Self::Host,
            "Origin" => Self::Origin,
            "Pragma" => Self::Pragma,
            "Referer" => Self::Referer,
            "User-Agent" => Self::UserAgent,
            "Via" => Self::Via,
            "Warning" => Self::Warning,
            custom => Self::Custom {
                name: custom.to_string(),
            },
        }
    }
}

impl Header for RequestHeader {
    fn default(&self) -> Option<&str> {
        match self {
            Self::Accept => Some("text/html"),
            Self::AcceptCharset => Some("utf-8"),
            Self::AcceptEncoding => Some("identity"),
            Self::AcceptLanguage => Some("en-GB"),
            Self::CacheControl => Some("no-cache"),
            Self::Connection => Some("close"),
            Self::ContentEncoding => Some("identity"),
            _ => None,
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

impl Header for ResponseHeader {
    fn default(&self) -> Option<&str> {
        match self {
            ResponseHeader::CacheControl => Some("max-age=3600"),
            ResponseHeader::Connection => Some("close"),
            ResponseHeader::ContentEncoding => Some("identity"),
            ResponseHeader::ContentLanguage => Some("en-GB"),
            ResponseHeader::ContentType => Some("text/html"),
            _ => None,
        }
    }
}

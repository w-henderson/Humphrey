use std::convert::TryFrom;

/// Represents an HTTP status code.
/// Can be converted to and from both `u16` and `&str`.
///
/// ## Example
/// ```
/// let status = StatusCode::NotFound;
/// let status2 = StatusCode::try_from(404)?;
/// assert_eq!(status, status2);
///
/// let status_code: u16 = status.into();
/// let status_name: &str = status.into();
/// assert_eq!(status_code, 404);
/// assert_eq!(status_name, "Not Found");
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StatusCode {
    Continue,
    SwitchingProtocols,
    OK,
    Created,
    Accepted,
    NonAuthoritative,
    NoContent,
    ResetContent,
    PartialContent,
    MultipleChoices,
    MovedPermanently,
    Found,
    SeeOther,
    NotModified,
    UseProxy,
    TemporaryRedirect,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    NotAcceptable,
    ProxyAuthenticationRequired,
    RequestTimeout,
    Conflict,
    Gone,
    LengthRequired,
    PreconditionFailed,
    RequestEntityTooLarge,
    RequestURITooLong,
    UnsupportedMediaType,
    RequestedRangeNotSatisfiable,
    ExpectationFailed,
    InternalError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
    GatewayTimeout,
    VersionNotSupported,
}

#[derive(PartialEq, Eq)]
pub struct StatusCodeError;

impl TryFrom<u16> for StatusCode {
    fn try_from(code: u16) -> Result<Self, StatusCodeError> {
        match code {
            100 => Ok(StatusCode::Continue),
            101 => Ok(StatusCode::SwitchingProtocols),
            200 => Ok(StatusCode::OK),
            201 => Ok(StatusCode::Created),
            202 => Ok(StatusCode::Accepted),
            203 => Ok(StatusCode::NonAuthoritative),
            204 => Ok(StatusCode::NoContent),
            205 => Ok(StatusCode::ResetContent),
            206 => Ok(StatusCode::PartialContent),
            300 => Ok(StatusCode::MultipleChoices),
            301 => Ok(StatusCode::MovedPermanently),
            302 => Ok(StatusCode::Found),
            303 => Ok(StatusCode::SeeOther),
            304 => Ok(StatusCode::NotModified),
            305 => Ok(StatusCode::UseProxy),
            307 => Ok(StatusCode::TemporaryRedirect),
            400 => Ok(StatusCode::BadRequest),
            401 => Ok(StatusCode::Unauthorized),
            403 => Ok(StatusCode::Forbidden),
            404 => Ok(StatusCode::NotFound),
            405 => Ok(StatusCode::MethodNotAllowed),
            406 => Ok(StatusCode::NotAcceptable),
            407 => Ok(StatusCode::ProxyAuthenticationRequired),
            408 => Ok(StatusCode::RequestTimeout),
            409 => Ok(StatusCode::Conflict),
            410 => Ok(StatusCode::Gone),
            411 => Ok(StatusCode::LengthRequired),
            412 => Ok(StatusCode::PreconditionFailed),
            413 => Ok(StatusCode::RequestEntityTooLarge),
            414 => Ok(StatusCode::RequestURITooLong),
            415 => Ok(StatusCode::UnsupportedMediaType),
            416 => Ok(StatusCode::RequestedRangeNotSatisfiable),
            417 => Ok(StatusCode::ExpectationFailed),
            500 => Ok(StatusCode::InternalError),
            501 => Ok(StatusCode::NotImplemented),
            502 => Ok(StatusCode::BadGateway),
            503 => Ok(StatusCode::ServiceUnavailable),
            504 => Ok(StatusCode::GatewayTimeout),
            505 => Ok(StatusCode::VersionNotSupported),
            _ => Err(StatusCodeError),
        }
    }

    type Error = StatusCodeError;
}

impl From<StatusCode> for u16 {
    fn from(val: StatusCode) -> Self {
        match val {
            StatusCode::Continue => 100,
            StatusCode::SwitchingProtocols => 101,
            StatusCode::OK => 200,
            StatusCode::Created => 201,
            StatusCode::Accepted => 202,
            StatusCode::NonAuthoritative => 203,
            StatusCode::NoContent => 204,
            StatusCode::ResetContent => 205,
            StatusCode::PartialContent => 206,
            StatusCode::MultipleChoices => 300,
            StatusCode::MovedPermanently => 301,
            StatusCode::Found => 302,
            StatusCode::SeeOther => 303,
            StatusCode::NotModified => 304,
            StatusCode::UseProxy => 305,
            StatusCode::TemporaryRedirect => 307,
            StatusCode::BadRequest => 400,
            StatusCode::Unauthorized => 401,
            StatusCode::Forbidden => 403,
            StatusCode::NotFound => 404,
            StatusCode::MethodNotAllowed => 405,
            StatusCode::NotAcceptable => 406,
            StatusCode::ProxyAuthenticationRequired => 407,
            StatusCode::RequestTimeout => 408,
            StatusCode::Conflict => 409,
            StatusCode::Gone => 410,
            StatusCode::LengthRequired => 411,
            StatusCode::PreconditionFailed => 412,
            StatusCode::RequestEntityTooLarge => 413,
            StatusCode::RequestURITooLong => 414,
            StatusCode::UnsupportedMediaType => 415,
            StatusCode::RequestedRangeNotSatisfiable => 416,
            StatusCode::ExpectationFailed => 417,
            StatusCode::InternalError => 500,
            StatusCode::NotImplemented => 501,
            StatusCode::BadGateway => 502,
            StatusCode::ServiceUnavailable => 503,
            StatusCode::GatewayTimeout => 504,
            StatusCode::VersionNotSupported => 505,
        }
    }
}

impl From<StatusCode> for &str {
    fn from(val: StatusCode) -> Self {
        match val {
            StatusCode::Continue => "Continue",
            StatusCode::SwitchingProtocols => "Switching Protocols",
            StatusCode::OK => "OK",
            StatusCode::Created => "Created",
            StatusCode::Accepted => "Accepted",
            StatusCode::NonAuthoritative => "Non-Authoritative Information",
            StatusCode::NoContent => "No Content",
            StatusCode::ResetContent => "Reset Content",
            StatusCode::PartialContent => "Partial Content",
            StatusCode::MultipleChoices => "Multiple Choices",
            StatusCode::MovedPermanently => "Moved Permanently",
            StatusCode::Found => "Found",
            StatusCode::SeeOther => "See Other",
            StatusCode::NotModified => "Not Modified",
            StatusCode::UseProxy => "Use Proxy",
            StatusCode::TemporaryRedirect => "Temporary Redirect",
            StatusCode::BadRequest => "Bad Request",
            StatusCode::Unauthorized => "Unauthorized",
            StatusCode::Forbidden => "Forbidden",
            StatusCode::NotFound => "Not Found",
            StatusCode::MethodNotAllowed => "Method Not Allowed",
            StatusCode::NotAcceptable => "Not Acceptable",
            StatusCode::ProxyAuthenticationRequired => "Proxy Authentication Required",
            StatusCode::RequestTimeout => "Request Timeout",
            StatusCode::Conflict => "Conflict",
            StatusCode::Gone => "Gone",
            StatusCode::LengthRequired => "Length Required",
            StatusCode::PreconditionFailed => "Precondition Failed",
            StatusCode::RequestEntityTooLarge => "Request Entity Too Large",
            StatusCode::RequestURITooLong => "Request-URI Too Long",
            StatusCode::UnsupportedMediaType => "Unsupported Media Type",
            StatusCode::RequestedRangeNotSatisfiable => "Requested Range Not Satisfiable",
            StatusCode::ExpectationFailed => "Expectation Failed",
            StatusCode::InternalError => "Internal Server Error",
            StatusCode::NotImplemented => "Not Implemented",
            StatusCode::BadGateway => "Bad Gateway",
            StatusCode::ServiceUnavailable => "Service Unavailable",
            StatusCode::GatewayTimeout => "Gateway Timeout",
            StatusCode::VersionNotSupported => "HTTP Version Not Supported",
        }
    }
}

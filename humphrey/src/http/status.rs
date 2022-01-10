//! Provides functionality for handling HTTP status codes.

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
    /// `100 Continue`: Continue with request.
    Continue,
    /// `101 Switching Protocols`: Protocol upgrade.
    SwitchingProtocols,
    /// `200 OK`: Request succeeded.
    OK,
    /// `201 Created`: Resource created.
    Created,
    /// `202 Accepted`: Request received, but not yet acted upon.
    Accepted,
    /// `203 Non-Authoritative Information`: Request processed, but response is from another source.
    NonAuthoritative,
    /// `204 No Content`: There is no content to send for this request.
    NoContent,
    /// `205 Reset Content`: Indicates that the document which sent this request should be reset.
    ResetContent,
    /// `206 Partial Content`: This response only contains part of a resource.
    PartialContent,
    /// `300 Multiple Choice`: The request has multiple possible responses.
    MultipleChoices,
    /// `301 Moved Permanently`: The resource has moved permanently to a new location.
    MovedPermanently,
    /// `302 Found`: The resource has moved temporarily to a new location.
    Found,
    /// `303 See Other`: The resource can be found under a different URI.
    SeeOther,
    /// `304 Not Modified`: The resource has not been modified since the last request.
    NotModified,
    /// `305 Use Proxy`: The requested resource must be accessed through a proxy.
    UseProxy,
    /// `307 Temporary Redirect`: The resource has moved temporarily to a new location.
    TemporaryRedirect,
    /// `400 Bad Request`: The request could not be understood by the server.
    BadRequest,
    /// `401 Unauthorized`: The request requires user authentication.
    Unauthorized,
    /// `403 Forbidden`: The client is not allowed to access this content.
    Forbidden,
    /// `404 Not Found`: The server can not find the requested resource.
    NotFound,
    /// `405 Method Not Allowed`: The method specified in the request is not allowed for the resource.
    MethodNotAllowed,
    /// `406 Not Acceptable`: No content that meets the criteria is available.
    NotAcceptable,
    /// `407 Proxy Authentication Required`: The client must first authenticate itself with a proxy.
    ProxyAuthenticationRequired,
    /// `408 Request Timeout`: The server timed out waiting for the request.
    RequestTimeout,
    /// `409 Conflict`: The request could not be completed because of a conflict with the server's current state.
    Conflict,
    /// `410 Gone`: The requested resource is no longer available.
    Gone,
    /// `411 Length Required`: The request did not specify the length of its content.
    LengthRequired,
    /// `412 Precondition Failed`: The server does not meet one of the client's preconditions.
    PreconditionFailed,
    /// `413 Payload Too Large`: The request is larger than the server is willing or able to process.
    RequestEntityTooLarge,
    /// `414 URI Too Long`: The URI provided was too long for the server to process.
    RequestURITooLong,
    /// `415 Unsupported Media Type`: The request entity has a media type which the server or resource does not support.
    UnsupportedMediaType,
    /// `416 Requested Range Not Satisfiable`: The range specified in the `Range` header cannot be fulfilled.
    RequestedRangeNotSatisfiable,
    /// `417 Expectation Failed`: The expectation given in the `Expect` header could not be met by the server.
    ExpectationFailed,
    /// `500 Internal Server Error`: The server encountered an unexpected error which prevented it from fulfilling the request.
    InternalError,
    /// `501 Not Implemented`: The server does not support the functionality required to fulfill the request.
    NotImplemented,
    /// `502 Bad Gateway`: The server, while acting as a gateway or proxy, received an invalid response from the upstream server.
    BadGateway,
    /// `503 Service Unavailable`: The server is temporarily unable to handle the request.
    ServiceUnavailable,
    /// `504 Gateway Timeout`: The server, while acting as a gateway or proxy, did not receive a timely response from the upstream server.
    GatewayTimeout,
    /// `505 HTTP Version Not Supported`: The server does not support the HTTP protocol version used in the request.
    VersionNotSupported,
}

/// Represents an error with the status code.
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

//! Defines traits for handler functions.

use crate::http::{Request, Response};
use crate::stream::Stream;

use std::sync::Arc;

/// Represents a function able to handle a WebSocket handshake and consequent data frames.
pub trait WebsocketHandler<State>: Send + Sync {
    #[allow(missing_docs)]
    fn serve(&self, request: Request, stream: Stream, state: Arc<State>);
}
impl<F, State> WebsocketHandler<State> for F
where
    F: Fn(Request, Stream, Arc<State>) + Send + Sync,
{
    fn serve(&self, request: Request, stream: Stream, state: Arc<State>) {
        self(request, stream, state)
    }
}

/// Represents a function able to handle a request.
/// It is passed the request as well as the app's state, and must return a response.
///
/// ## Example
/// The most basic request handler would be as follows:
/// ```
/// fn handler(_: Request, _: Arc<()>) -> Response {
///     Response::new(StatusCode::OK, b"Success")
/// }
/// ```
pub trait RequestHandler<State>: Send + Sync {
    #[allow(missing_docs)]
    fn serve(&self, request: Request, state: Arc<State>) -> Response;
}
impl<F, State> RequestHandler<State> for F
where
    F: Fn(Request, Arc<State>) -> Response + Send + Sync,
{
    fn serve(&self, request: Request, state: Arc<State>) -> Response {
        self(request, state)
    }
}

/// Represents a function able to handle a request.
/// It is passed only the request, and must return a response.
/// If you want access to the app's state, consider using the `RequestHandler` trait instead.
///
/// ## Example
/// The most basic stateless request handler would be as follows:
/// ```
/// fn handler(_: Request) -> Response {
///     Response::new(StatusCode::OK, b"Success")
/// }
/// ```
pub trait StatelessRequestHandler<State>: Send + Sync {
    #[allow(missing_docs)]
    fn serve(&self, request: Request) -> Response;
}
impl<F, State> StatelessRequestHandler<State> for F
where
    F: Fn(Request) -> Response + Send + Sync,
{
    fn serve(&self, request: Request) -> Response {
        self(request)
    }
}

/// Represents a function able to handle a request with respect to the route it was called from.
/// It is passed the request, the app's state, and the route it was called from, and must return a response.
///
/// ## Example
/// The most basic path-aware request handler would be as follows:
/// ```
/// fn handler(_: Request, _: Arc<()>, route: &str) -> Response {
///     Response::new(StatusCode::OK, format!("Success matching route {}", route))
/// }
/// ```
pub trait PathAwareRequestHandler<State>: Send + Sync {
    #[allow(missing_docs)]
    fn serve(&self, request: Request, state: Arc<State>, route: &'static str) -> Response;
}
impl<F, State> PathAwareRequestHandler<State> for F
where
    F: Fn(Request, Arc<State>, &'static str) -> Response + Send + Sync,
{
    fn serve(&self, request: Request, state: Arc<State>, route: &'static str) -> Response {
        self(request, state, route)
    }
}

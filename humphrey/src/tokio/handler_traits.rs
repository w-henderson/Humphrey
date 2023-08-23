use crate::http::{Request, Response};
use crate::stream::Stream;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Represents a function able to handle a WebSocket handshake and consequent data frames.
pub trait WebsocketHandler<State>: Send + Sync {
    #[allow(missing_docs)]
    fn serve(
        &self,
        request: Request,
        stream: Stream,
        state: Arc<State>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}
impl<F, Fut, State> WebsocketHandler<State> for F
where
    F: Fn(Request, Stream, Arc<State>) -> Fut + Send + Sync,
    Fut: Future<Output = ()> + Send + 'static,
{
    fn serve(
        &self,
        request: Request,
        stream: Stream,
        state: Arc<State>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(self(request, stream, state))
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
    fn serve(
        &self,
        request: Request,
        state: Arc<State>,
    ) -> Pin<Box<dyn Future<Output = Response> + Send>>;
}
impl<F, Fut, State> RequestHandler<State> for F
where
    F: Fn(Request, Arc<State>) -> Fut + Send + Sync,
    Fut: Future<Output = Response> + Send + 'static,
{
    fn serve(
        &self,
        request: Request,
        state: Arc<State>,
    ) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        Box::pin(self(request, state))
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
    fn serve(&self, request: Request) -> Pin<Box<dyn Future<Output = Response> + Send>>;
}
impl<F, Fut, State> StatelessRequestHandler<State> for F
where
    F: Fn(Request) -> Fut + Send + Sync,
    Fut: Future<Output = Response> + Send + 'static,
{
    fn serve(&self, request: Request) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        Box::pin(self(request))
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
    fn serve(
        &self,
        request: Request,
        state: Arc<State>,
        route: &'static str,
    ) -> Pin<Box<dyn Future<Output = Response> + Send>>;
}
impl<F, Fut, State> PathAwareRequestHandler<State> for F
where
    F: Fn(Request, Arc<State>, &'static str) -> Fut + Send + Sync,
    Fut: Future<Output = Response> + Send + 'static,
{
    fn serve(
        &self,
        request: Request,
        state: Arc<State>,
        route: &'static str,
    ) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        Box::pin(self(request, state, route))
    }
}

#![allow(clippy::new_without_default)]

use crate::http::headers::RequestHeader;
use crate::http::request::{Request, RequestError};
use crate::http::response::Response;
use crate::http::status::StatusCode;
use crate::route::{Route, RouteHandler};
use crate::thread::pool::ThreadPool;

use std::io::Write;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::Arc;

/// Represents the Humphrey app.
///
/// The type parameter represents the app state, which is shared between threads.
/// It must implement the `Send` and `Sync` traits to be sent between threads.
/// The state is given to every request as an `Arc<State>`.
pub struct App<State = ()>
where
    State: Send + Sync + 'static,
{
    thread_pool: ThreadPool,
    routes: Vec<RouteHandler<State>>,
    error_handler: ErrorHandler,
    state: Arc<State>,
    connection_handler: ConnectionHandler<State>,
    connection_condition: ConnectionCondition<State>,
    websocket_handler: Arc<dyn WebsocketHandler<State>>,
}

/// Represents a function able to handle a connection.
/// In most cases, the default connection handler should be used.
pub type ConnectionHandler<State> = fn(
    TcpStream,
    Arc<Vec<RouteHandler<State>>>,
    Arc<ErrorHandler>,
    Arc<dyn WebsocketHandler<State>>,
    Arc<State>,
);

/// Represents a function able to calculate whether a connection will be accepted.
pub type ConnectionCondition<State> = fn(&mut TcpStream, Arc<State>) -> bool;

/// Represents a function able to handle a WebSocket handshake and consequent data frames.
pub trait WebsocketHandler<State>: Fn(Request, TcpStream, Arc<State>) + Send + Sync {}
impl<T, S> WebsocketHandler<S> for T where T: Fn(Request, TcpStream, Arc<S>) + Send + Sync {}

/// Represents a function able to handle a request.
/// It is passed the request as well as the app's state, and must return a response.
///
/// ## Example
/// The most basic request handler would be as follows:
/// ```
/// fn handler(request: Request, _: Arc<()>) -> Response {
///     Response::new(StatusCode::OK, b"Success", &request)
/// }
/// ```
pub trait RequestHandler<State>: Fn(Request, Arc<State>) -> Response + Send + Sync {}
impl<T, S> RequestHandler<S> for T where T: Fn(Request, Arc<S>) -> Response + Send + Sync {}

/// Represents a function able to handle a request.
/// It is passed only the request, and must return a response.
/// If you want access to the app's state, consider using the `RequestHandler` trait instead.
///
/// ## Example
/// The most basic stateless request handler would be as follows:
/// ```
/// fn handler(request: Request) -> Response {
///     Response::new(StatusCode::OK, b"Success", &request)
/// }
/// ```
pub trait StatelessRequestHandler<State>: Fn(Request) -> Response + Send + Sync {}
impl<T, S> StatelessRequestHandler<S> for T where T: Fn(Request) -> Response + Send + Sync {}

/// Represents a function able to handle a request with respect to the route it was called from.
/// It is passed the request, the app's state, and the route it was called from, and must return a response.
///
/// ## Example
/// The most basic path-aware request handler would be as follows:
/// ```
/// fn handler(request: Request, _: Arc<()>, route: &str) -> Response {
///     Response::new(StatusCode::OK, format!("Success matching route {}", route), &request)
/// }
/// ```
#[rustfmt::skip]
pub trait PathAwareRequestHandler<State>:
    Fn(Request, Arc<State>, &str) -> Response + Send + Sync {}
#[rustfmt::skip]
impl<T, S> PathAwareRequestHandler<S> for T where
    T: Fn(Request, Arc<S>, &str) -> Response + Send + Sync {}

/// Represents a function able to handle an error.
/// The first parameter of type `Option<Request>` will be `Some` if the request could be parsed.
/// Otherwise, it will be `None` and the status code will be `StatusCode::BadRequest`.
///
/// Every app has a default error handler, which simply displays the status code.
/// The source code for this default error handler is copied below since it is a good example.
///
/// ## Example
/// ```
/// fn error_handler(request: Option<Request>, status_code: StatusCode) -> Response {
///     let body = format!(
///         "<html><body><h1>{} {}</h1></body></html>",
///         Into::<u16>::into(status_code.clone()),
///         Into::<&str>::into(status_code.clone())
///     );
///     
///     if let Some(request) = request {
///         Response::new(status_code, body.as_bytes(), &request)
///     } else {
///         Response::empty(status_code)
///             .with_bytes(body.as_bytes())
///             .with_generated_headers()
///     }
/// }
/// ```
pub type ErrorHandler = fn(Option<Request>, StatusCode) -> Response;

/// Represents a generic error with the program.
pub type HumphreyError = Box<dyn std::error::Error>;

impl<State> App<State>
where
    State: Send + Sync + 'static,
{
    /// Initialises a new Humphrey app.
    ///
    /// Initialising an app like this requires the app state type to implement `Default` in order to
    ///   automatically generate an initial value for the state. If this requirement is not, or cannnot
    ///   be met, please use `App::new_with_config` and specify a number of threads and the default
    ///   state value.
    pub fn new() -> Self
    where
        State: Default,
    {
        Self {
            thread_pool: ThreadPool::new(32),
            routes: Vec::new(),
            error_handler,
            state: Arc::new(State::default()),
            connection_handler: client_handler,
            connection_condition: |_, _| true,
            websocket_handler: Arc::new(|_, _, _| ()),
        }
    }

    /// Initialises a new Humphrey app with the given configuration options.
    pub fn new_with_config(threads: usize, state: State) -> Self {
        Self {
            thread_pool: ThreadPool::new(threads),
            routes: Vec::new(),
            error_handler,
            state: Arc::new(state),
            connection_handler: client_handler,
            connection_condition: |_, _| true,
            websocket_handler: Arc::new(|_, _, _| ()),
        }
    }

    /// Runs the Humphrey app on the given socket address.
    /// This function will only return if a fatal error is thrown such as the port being in use.
    pub fn run<A>(self, addr: A) -> Result<(), HumphreyError>
    where
        A: ToSocketAddrs,
    {
        let socket = TcpListener::bind(addr)?;
        let routes = Arc::new(self.routes);
        let error_handler = Arc::new(self.error_handler);

        for mut stream in socket.incoming().flatten() {
            let cloned_state = self.state.clone();

            // Check that the client is allowed to connect
            if (self.connection_condition)(&mut stream, cloned_state) {
                let cloned_state = self.state.clone();
                let cloned_routes = routes.clone();
                let cloned_websocket_handler = self.websocket_handler.clone();
                let cloned_error_handler = error_handler.clone();
                let cloned_handler = self.connection_handler;

                // Spawn a new thread to handle the connection
                self.thread_pool.execute(move || {
                    (cloned_handler)(
                        stream,
                        cloned_routes,
                        cloned_error_handler,
                        cloned_websocket_handler,
                        cloned_state,
                    )
                });
            }
        }

        Ok(())
    }

    /// Sets the default state for the server.
    /// Should only be used in cases where the `Default` trait cannot be implemented for `State`.
    /// For example, if the default state is dynamically generated as it is in the CLI.
    pub fn with_state(mut self, state: State) -> Self {
        self.state = Arc::new(state);
        self
    }

    /// Adds a route and associated handler to the server.
    /// Routes can include wildcards, for example `/blog/*`.
    ///
    /// ## Panics
    /// This function will panic if the route string cannot be converted to a `Uri` object.
    pub fn with_route<T>(mut self, route: &str, handler: T) -> Self
    where
        T: RequestHandler<State> + 'static,
    {
        self.routes.push(RouteHandler {
            route: route.parse().unwrap(),
            handler: Box::new(handler),
        });
        self
    }

    /// Adds a route and associated handler to the server.
    /// Does not pass the state to the handler.
    /// Routes can include wildcards, for example `/blog/*`.
    ///
    /// If you want to access the app's state in the handler, consider using `with_route`.
    ///
    /// ## Panics
    /// This function will panic if the route string cannot be converted to a `Uri` object.
    pub fn with_stateless_route<T>(mut self, route: &str, handler: T) -> Self
    where
        T: StatelessRequestHandler<State> + 'static,
    {
        self.routes.push(RouteHandler {
            route: route.parse().unwrap(),
            handler: Box::new(move |request, _| handler(request)),
        });
        self
    }

    /// Adds a path-aware route and associated handler to the server.
    /// Routes can include wildcards, for example `/blog/*`.
    /// Will also pass the route to the handler at runtime.
    ///
    /// ## Panics
    /// This function will panic if the route string cannot be converted to a `Uri` object.
    pub fn with_path_aware_route<T>(mut self, route: &'static str, handler: T) -> Self
    where
        T: PathAwareRequestHandler<State> + 'static,
    {
        self.routes.push(RouteHandler {
            route: route.parse().unwrap(),
            handler: Box::new(move |request, state| handler(request, state, route)),
        });
        self
    }

    /// Sets the error handler for the server.
    pub fn with_error_handler(mut self, handler: ErrorHandler) -> Self {
        self.error_handler = handler;
        self
    }

    /// Sets the connection condition, a function which decides whether to accept the connection.
    /// For example, this could be used for implementing whitelists and blacklists.
    pub fn with_connection_condition(mut self, condition: ConnectionCondition<State>) -> Self {
        self.connection_condition = condition;
        self
    }

    /// Sets the websocket handler, a function which processes WebSocket handshakes.
    /// This is passed the stream, state, and the request which triggered its calling.
    pub fn with_websocket_handler<T>(mut self, handler: T) -> Self
    where
        T: WebsocketHandler<State> + 'static,
    {
        self.websocket_handler = Arc::new(handler);
        self
    }

    /// Overrides the default connection handler, allowing for manual control over the TCP requests and responses.
    /// Not recommended as it basically disables most of the server's features.
    pub fn with_custom_connection_handler(mut self, handler: ConnectionHandler<State>) -> Self {
        self.connection_handler = handler;
        self
    }

    /// Gets a reference to the app's state.
    /// This should only be used in the main thread, as the state is passed to request handlers otherwise.
    pub fn get_state(&self) -> Arc<State> {
        self.state.clone()
    }
}

/// Handles a connection with a client.
/// The connection will be opened upon the first request and closed as soon as a request is
///   recieved without the `Connection: Keep-Alive` header.
fn client_handler<State>(
    mut stream: TcpStream,
    routes: Arc<Vec<RouteHandler<State>>>,
    error_handler: Arc<ErrorHandler>,
    websocket_handler: Arc<dyn WebsocketHandler<State>>,
    state: Arc<State>,
) {
    let addr = stream.peer_addr().unwrap();

    loop {
        // Parses the request from the stream
        let request = Request::from_stream(&mut stream, addr);
        let cloned_state = state.clone();

        // If the request is valid an is a websocket request, call the corresponding handler
        if let Ok(req) = &request {
            if req.headers.get(&RequestHeader::Upgrade) == Some(&"websocket".to_string()) {
                (websocket_handler)(req.clone(), stream, cloned_state);
                break;
            }
        }

        // If the request could not be parsed due to a stream error, close the thread
        if match &request {
            Ok(_) => false,
            Err(e) => e == &RequestError::Stream,
        } {
            break;
        }

        // Get the keep alive information from the request before it is consumed by the handler
        let keep_alive = if let Ok(request) = &request {
            if let Some(connection) = request.headers.get(&RequestHeader::Connection) {
                connection.to_ascii_lowercase() == "keep-alive"
            } else {
                false
            }
        } else {
            false
        };

        // Generate the response based on the handlers
        let response = match request {
            Ok(request) => match routes.iter().find(|r| r.route.route_matches(&request.uri)) {
                Some(handler) => (handler.handler)(request, cloned_state),
                None => error_handler(Some(request), StatusCode::NotFound),
            },
            Err(_) => error_handler(None, StatusCode::BadRequest),
        };

        // Write the response to the stream
        let response_bytes: Vec<u8> = response.into();
        if stream.write(&response_bytes).is_err() {
            break;
        };

        // If the request specified to keep the connection open, respect this
        if !keep_alive {
            break;
        }
    }
}

/// The default error handler for every Humphrey app.
/// This can be overridden by using the `with_error_handler` method when building the app.
pub(crate) fn error_handler(request: Option<Request>, status_code: StatusCode) -> Response {
    let body = format!(
        "<html><body><h1>{} {}</h1></body></html>",
        Into::<u16>::into(status_code.clone()),
        Into::<&str>::into(status_code.clone())
    );

    if let Some(request) = request {
        Response::new(status_code, body.as_bytes(), &request)
    } else {
        Response::empty(status_code)
            .with_bytes(body.as_bytes().to_vec())
            .with_generated_headers()
    }
}

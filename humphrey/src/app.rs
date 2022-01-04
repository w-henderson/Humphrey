#![allow(clippy::new_without_default)]

use crate::http::headers::RequestHeader;
use crate::http::request::Request;
use crate::http::response::Response;
use crate::http::status::StatusCode;
use crate::krauss::wildcard_match;
use crate::route::{Route, SubApp};
use crate::thread::pool::ThreadPool;

#[cfg(feature = "tls")]
use crate::tls;

use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::Arc;

#[cfg(feature = "tls")]
use rustls::{ServerConfig, ServerConnection, Stream};

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
    subapps: Vec<SubApp<State>>,
    default_subapp: SubApp<State>,
    error_handler: ErrorHandler,
    state: Arc<State>,
    connection_handler: ConnectionHandler<State>,
    connection_condition: ConnectionCondition<State>,
}

/// Represents a function able to handle a connection.
/// In most cases, the default connection handler should be used.
#[cfg(feature = "tls")]
pub type ConnectionHandler<State> = fn(
    TcpStream,
    Arc<Vec<SubApp<State>>>,
    Arc<SubApp<State>>,
    Arc<ErrorHandler>,
    Arc<State>,
    Arc<ServerConfig>,
);
#[cfg(not(feature = "tls"))]
pub type ConnectionHandler<State> =
    fn(TcpStream, Arc<Vec<SubApp<State>>>, Arc<SubApp<State>>, Arc<ErrorHandler>, Arc<State>);

/// Represents a function able to calculate whether a connection will be accepted.
pub type ConnectionCondition<State> = fn(&mut TcpStream, Arc<State>) -> bool;

/// Represents a function able to handle a WebSocket handshake and consequent data frames.
#[cfg(feature = "tls")]
pub trait WebsocketHandler<State>:
    Fn(Request, Stream<ServerConnection, TcpStream>, Arc<State>) + Send + Sync
{
}
#[cfg(not(feature = "tls"))]
pub trait WebsocketHandler<State>: Fn(Request, TcpStream, Arc<State>) + Send + Sync {}

#[cfg(feature = "tls")]
impl<T, S> WebsocketHandler<S> for T where
    T: Fn(Request, Stream<ServerConnection, TcpStream>, Arc<S>) + Send + Sync
{
}
#[cfg(not(feature = "tls"))]
impl<T, S> WebsocketHandler<S> for T where T: Fn(Request, TcpStream, Arc<S>) + Send + Sync {}

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
pub trait RequestHandler<State>: Fn(Request, Arc<State>) -> Response + Send + Sync {}
impl<T, S> RequestHandler<S> for T where T: Fn(Request, Arc<S>) -> Response + Send + Sync {}

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
pub trait StatelessRequestHandler<State>: Fn(Request) -> Response + Send + Sync {}
impl<T, S> StatelessRequestHandler<S> for T where T: Fn(Request) -> Response + Send + Sync {}

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
/// fn error_handler(status_code: StatusCode) -> Response {
///     let body = format!(
///         "<html><body><h1>{} {}</h1></body></html>",
///         Into::<u16>::into(status_code.clone()),
///         Into::<&str>::into(status_code.clone())
///     );
///     
///     Response::new(status_code, body.as_bytes())
/// }
/// ```
pub type ErrorHandler = fn(StatusCode) -> Response;

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
            subapps: Vec::new(),
            default_subapp: SubApp::default(),
            error_handler,
            state: Arc::new(State::default()),
            #[cfg(not(feature = "tls"))]
            connection_handler: client_handler,
            #[cfg(feature = "tls")]
            connection_handler: tls::client_handler,
            connection_condition: |_, _| true,
        }
    }

    /// Initialises a new Humphrey app with the given configuration options.
    pub fn new_with_config(threads: usize, state: State) -> Self {
        Self {
            thread_pool: ThreadPool::new(threads),
            subapps: Vec::new(),
            default_subapp: SubApp::default(),
            error_handler,
            state: Arc::new(state),
            #[cfg(not(feature = "tls"))]
            connection_handler: client_handler,
            #[cfg(feature = "tls")]
            connection_handler: tls::client_handler,
            connection_condition: |_, _| true,
        }
    }

    /// Runs the Humphrey app on the given socket address.
    /// This function will only return if a fatal error is thrown such as the port being in use.
    #[cfg(not(feature = "tls"))]
    pub fn run<A>(self, addr: A) -> Result<(), HumphreyError>
    where
        A: ToSocketAddrs,
    {
        let socket = TcpListener::bind(addr)?;
        let subapps = Arc::new(self.subapps);
        let default_subapp = Arc::new(self.default_subapp);
        let error_handler = Arc::new(self.error_handler);

        for mut stream in socket.incoming().flatten() {
            let cloned_state = self.state.clone();

            // Check that the client is allowed to connect
            if (self.connection_condition)(&mut stream, cloned_state) {
                let cloned_state = self.state.clone();
                let cloned_subapps = subapps.clone();
                let cloned_default_subapp = default_subapp.clone();
                let cloned_error_handler = error_handler.clone();
                let cloned_handler = self.connection_handler;

                // Spawn a new thread to handle the connection
                self.thread_pool.execute(move || {
                    (cloned_handler)(
                        stream,
                        cloned_subapps,
                        cloned_default_subapp,
                        cloned_error_handler,
                        cloned_state,
                    )
                });
            }
        }

        Ok(())
    }

    #[cfg(feature = "tls")]
    pub fn run<A>(self, addr: A) -> Result<(), HumphreyError>
    where
        A: ToSocketAddrs,
    {
        use rustls::{Certificate, PrivateKey, ServerConfig, ServerSession};
        use rustls_pemfile::{read_one, Item};

        use std::fs::File;
        use std::io::BufReader;

        let socket = TcpListener::bind(addr)?;
        let subapps = Arc::new(self.subapps);
        let default_subapp = Arc::new(self.default_subapp);
        let error_handler = Arc::new(self.error_handler);

        let mut cert_file = BufReader::new(File::open("keys/localhost.pem")?);
        let mut key_file = BufReader::new(File::open("keys/localhost-key.pem")?);

        let certs: Vec<Certificate> = match read_one(&mut cert_file)?.unwrap() {
            Item::X509Certificate(cert) => vec![Certificate(cert)],
            _ => panic!("not pog cert"),
        };

        let key: PrivateKey = match read_one(&mut key_file)?.unwrap() {
            Item::PKCS8Key(key) => PrivateKey(key),
            _ => panic!("not pog key"),
        };

        let config = Arc::new(
            ServerConfig::builder()
                .with_safe_defaults()
                .with_no_client_auth()
                .with_single_cert(certs, key)?,
        );

        for mut stream in socket.incoming().flatten() {
            let cloned_state = self.state.clone();

            // Check that the client is allowed to connect
            if (self.connection_condition)(&mut stream, cloned_state) {
                let cloned_state = self.state.clone();
                let cloned_subapps = subapps.clone();
                let cloned_default_subapp = default_subapp.clone();
                let cloned_error_handler = error_handler.clone();
                let cloned_handler = self.connection_handler;
                let cloned_config = config.clone();

                // Spawn a new thread to handle the connection
                self.thread_pool.execute(move || {
                    (cloned_handler)(
                        stream,
                        cloned_subapps,
                        cloned_default_subapp,
                        cloned_error_handler,
                        cloned_state,
                        cloned_config,
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

    /// Adds a new host sub-app to the server.
    /// The host can contain wildcards, for example `*.example.com`.
    ///
    /// ## Panics
    /// This function will panic if the host is equal to `*`, since this is the default host.
    /// If you want to add a route to every host, simply add it directly to the main app.
    pub fn with_host(mut self, host: &str, mut handler: SubApp<State>) -> Self {
        if host == "*" {
            panic!("Cannot add a sub-app with wildcard `*`");
        }

        handler.host = host.to_string();
        self.subapps.push(handler);

        self
    }

    /// Adds a route and associated handler to the server.
    /// Routes can include wildcards, for example `/blog/*`.
    pub fn with_route<T>(mut self, route: &str, handler: T) -> Self
    where
        T: RequestHandler<State> + 'static,
    {
        self.default_subapp = self.default_subapp.with_route(route, handler);
        self
    }

    /// Adds a route and associated handler to the server.
    /// Does not pass the state to the handler.
    /// Routes can include wildcards, for example `/blog/*`.
    ///
    /// If you want to access the app's state in the handler, consider using `with_route`.
    pub fn with_stateless_route<T>(mut self, route: &str, handler: T) -> Self
    where
        T: StatelessRequestHandler<State> + 'static,
    {
        self.default_subapp = self.default_subapp.with_stateless_route(route, handler);
        self
    }

    /// Adds a path-aware route and associated handler to the server.
    /// Routes can include wildcards, for example `/blog/*`.
    /// Will also pass the route to the handler at runtime.
    pub fn with_path_aware_route<T>(mut self, route: &'static str, handler: T) -> Self
    where
        T: PathAwareRequestHandler<State> + 'static,
    {
        self.default_subapp = self.default_subapp.with_path_aware_route(route, handler);
        self
    }

    /// Adds a WebSocket route and associated handler to the server.
    /// Routes can include wildcards, for example `/ws/*`.
    /// The handler is passed the stream, state, and the request which triggered its calling.
    pub fn with_websocket_route<T>(mut self, route: &str, handler: T) -> Self
    where
        T: WebsocketHandler<State> + 'static,
    {
        self.default_subapp = self.default_subapp.with_websocket_route(route, handler);
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

    /// Adds a global WebSocket handler to the server.
    ///
    /// ## Deprecated
    /// This function is deprecated and will be removed in a future version.
    /// Please use `with_websocket_route` intead.
    #[deprecated(since = "0.3.0", note = "Please use `with_websocket_route` instead")]
    pub fn with_websocket_handler<T>(mut self, handler: T) -> Self
    where
        T: WebsocketHandler<State> + 'static,
    {
        self.default_subapp = self.default_subapp.with_websocket_route("*", handler);
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
#[cfg(not(feature = "tls"))]
fn client_handler<State>(
    mut stream: TcpStream,
    subapps: Arc<Vec<SubApp<State>>>,
    default_subapp: Arc<SubApp<State>>,
    error_handler: Arc<ErrorHandler>,
    state: Arc<State>,
) {
    use std::collections::btree_map::Entry;
    use std::io::Write;

    use crate::http::date::DateTime;
    use crate::http::headers::ResponseHeader;
    use crate::http::request::RequestError;

    let addr = stream.peer_addr().unwrap();

    loop {
        // Parses the request from the stream
        let request = Request::from_stream(&mut stream, addr);
        let cloned_state = state.clone();

        // If the request is valid an is a WebSocket request, call the corresponding handler
        if let Ok(req) = &request {
            if req.headers.get(&RequestHeader::Upgrade) == Some(&"websocket".to_string()) {
                call_websocket_handler(req, &subapps, &default_subapp, cloned_state, stream);
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
            Ok(request) => {
                let mut response = call_handler(&request, &subapps, &default_subapp, state.clone());

                // Automatically generate required headers
                match response.headers.entry(ResponseHeader::Connection) {
                    Entry::Occupied(_) => (),
                    Entry::Vacant(v) => {
                        if let Some(connection) = &request.headers.get(&RequestHeader::Connection) {
                            v.insert(connection.to_string());
                        } else {
                            v.insert("Close".to_string());
                        }
                    }
                }

                match response.headers.entry(ResponseHeader::Server) {
                    Entry::Occupied(_) => (),
                    Entry::Vacant(v) => {
                        v.insert("Humphrey".to_string());
                    }
                }

                match response.headers.entry(ResponseHeader::Date) {
                    Entry::Occupied(_) => (),
                    Entry::Vacant(v) => {
                        v.insert(DateTime::now().to_string());
                    }
                }

                match response.headers.entry(ResponseHeader::ContentLength) {
                    Entry::Occupied(_) => (),
                    Entry::Vacant(v) => {
                        v.insert(response.body.len().to_string());
                    }
                }

                // Set HTTP version
                response.version = request.version.clone();

                response
            }
            Err(_) => error_handler(StatusCode::BadRequest),
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

/// Calls the correct handler for the given request.
pub(crate) fn call_handler<State>(
    request: &Request,
    subapps: &[SubApp<State>],
    default_subapp: &SubApp<State>,
    state: Arc<State>,
) -> Response {
    let host = request.headers.get(&RequestHeader::Host).unwrap();

    // Iterate over the sub-apps and find the one which matches the host
    if let Some(subapp) = subapps
        .iter()
        .find(|subapp| wildcard_match(&subapp.host, host))
    {
        // If the sub-app has a handler for this route, call it
        if let Some(response) = subapp
            .routes // Get the routes of the sub-app
            .iter() // Iterate over the routes
            .find(|route| route.route.route_matches(&request.uri)) // Find the route that matches
            .map(|handler| (handler.handler)(request.clone(), state.clone()))
        {
            return response;
        }
    }

    // If no sub-app was found, try to use the handler on the default sub-app
    if let Some(response) = default_subapp
        .routes
        .iter()
        .find(|route| route.route.route_matches(&request.uri))
        .map(|handler| (handler.handler)(request.clone(), state))
    {
        return response;
    }

    // Otherwise return an error
    error_handler(StatusCode::NotFound)
}

/// Calls the correct WebSocket handler for the given request.
#[cfg(not(feature = "tls"))]
fn call_websocket_handler<State>(
    request: &Request,
    subapps: &[SubApp<State>],
    default_subapp: &SubApp<State>,
    state: Arc<State>,
    stream: TcpStream,
) {
    let host = request.headers.get(&RequestHeader::Host).unwrap();

    // Iterate over the sub-apps and find the one which matches the host
    if let Some(subapp) = subapps
        .iter()
        .find(|subapp| wildcard_match(&subapp.host, host))
    {
        // If the sub-app has a handler for this route, call it
        if let Some(handler) = subapp
            .websocket_routes // Get the WebSocket routes of the sub-app
            .iter() // Iterate over the routes
            .find(|route| route.route.route_matches(&request.uri))
        {
            (handler.handler)(request.clone(), stream, state);
            return;
        }
    }

    // If no sub-app was found, try to use the handler on the default sub-app
    if let Some(handler) = default_subapp
        .websocket_routes
        .iter()
        .find(|route| route.route.route_matches(&request.uri))
    {
        (handler.handler)(request.clone(), stream, state)
    }
}

/// The default error handler for every Humphrey app.
/// This can be overridden by using the `with_error_handler` method when building the app.
pub(crate) fn error_handler(status_code: StatusCode) -> Response {
    let body = format!(
        "<html><body><h1>{} {}</h1></body></html>",
        Into::<u16>::into(status_code.clone()),
        Into::<&str>::into(status_code.clone())
    );

    Response::new(status_code, body.as_bytes())
}

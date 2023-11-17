//! Provides the core Humphrey app functionality.

#![allow(clippy::new_without_default)]

use crate::http::cors::Cors;
use crate::http::date::DateTime;
use crate::http::headers::HeaderType;
use crate::http::method::Method;
use crate::http::request::{Request, RequestError};
use crate::http::response::Response;
use crate::http::status::StatusCode;
use crate::krauss::wildcard_match;
use crate::monitor::event::{Event, EventType};
use crate::monitor::MonitorConfig;
use crate::route::{Route, RouteHandler, SubApp};
use crate::stream::Stream;
use crate::thread::pool::ThreadPool;

use std::io::Write;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;

use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "tls")]
use rustls::ServerConfig;

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
    monitor: MonitorConfig,
    connection_handler: ConnectionHandler<State>,
    connection_condition: ConnectionCondition<State>,
    connection_timeout: Option<Duration>,
    shutdown: Option<Arc<AtomicBool>>,
    #[cfg(feature = "tls")]
    tls_config: Option<Arc<ServerConfig>>,
    #[cfg(feature = "tls")]
    force_https: bool,
}

/// Represents a function able to handle a connection.
/// In most cases, the default connection handler should be used.
pub type ConnectionHandler<State> = fn(
    Stream,
    Arc<Vec<SubApp<State>>>,
    Arc<SubApp<State>>,
    Arc<ErrorHandler>,
    Arc<State>,
    MonitorConfig,
    Option<Duration>,
);

/// Represents a function able to calculate whether a connection will be accepted.
pub type ConnectionCondition<State> = fn(&mut TcpStream, Arc<State>) -> bool;

pub use crate::handler_traits::*;

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
            monitor: MonitorConfig::default(),
            connection_handler: client_handler,
            connection_condition: |_, _| true,
            connection_timeout: None,
            shutdown: None,
            #[cfg(feature = "tls")]
            tls_config: None,
            #[cfg(feature = "tls")]
            force_https: false,
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
            monitor: MonitorConfig::default(),
            connection_handler: client_handler,
            connection_condition: |_, _| true,
            connection_timeout: None,
            shutdown: None,
            #[cfg(feature = "tls")]
            tls_config: None,
            #[cfg(feature = "tls")]
            force_https: false,
        }
    }

    /// Runs the Humphrey app on the given socket address.
    /// This function will only return if a fatal error is thrown such as the port being in use.
    pub fn run<A>(mut self, addr: A) -> Result<(), HumphreyError>
    where
        A: ToSocketAddrs,
    {
        let socket = TcpListener::bind(addr)?;

        if self.shutdown.is_some() {
            socket.set_nonblocking(true).expect("Cannot set non-blocking");
        }

        let subapps = Arc::new(self.subapps);
        let default_subapp = Arc::new(self.default_subapp);
        let error_handler = Arc::new(self.error_handler);

        self.thread_pool.register_monitor(self.monitor.clone());
        self.thread_pool.start();

        for stream in socket.incoming() {
            match stream {
                Ok(mut stream) => {
                    let cloned_state = self.state.clone();

                    // Check that the client is allowed to connect
                    if (self.connection_condition)(&mut stream, cloned_state) {
                        let cloned_state = self.state.clone();
                        let cloned_monitor = self.monitor.clone();
                        let cloned_subapps = subapps.clone();
                        let cloned_default_subapp = default_subapp.clone();
                        let cloned_error_handler = error_handler.clone();
                        let cloned_handler = self.connection_handler;
                        let cloned_timeout = self.connection_timeout;

                        cloned_monitor.send(
                            Event::new(EventType::ConnectionSuccess)
                                .with_peer_result(stream.peer_addr()),
                        );

                        // Spawn a new thread to handle the connection
                        self.thread_pool.execute(move || {
                            cloned_monitor.send(
                                Event::new(EventType::ThreadPoolProcessStarted)
                                    .with_peer_result(stream.peer_addr()),
                            );

                            (cloned_handler)(
                                Stream::Tcp(stream),
                                cloned_subapps,
                                cloned_default_subapp,
                                cloned_error_handler,
                                cloned_state,
                                cloned_monitor,
                                cloned_timeout,
                            )
                        });
                    } else {
                        self.monitor.send(
                            Event::new(EventType::ConnectionDenied)
                                .with_peer_result(stream.peer_addr()),
                        );
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if let Some(ref shutdown) = self.shutdown {
                        if shutdown.load(Ordering::Relaxed) { println!("got shutdown"); break }
                    }
                },
                Err(e) => self
                    .monitor
                    .send(Event::new(EventType::ConnectionError).with_info(e.to_string())),
            }
        }

        Ok(())
    }

    /// Securely runs the Humphrey app on the given socket address.
    /// This function will only return if a fatal error is thrown such as the port being in use or the TLS certificate being invalid.
    #[cfg(feature = "tls")]
    pub fn run_tls<A>(mut self, addr: A) -> Result<(), HumphreyError>
    where
        A: ToSocketAddrs,
    {
        use rustls::ServerConnection;

        let socket = TcpListener::bind(addr)?;
        let subapps = Arc::new(self.subapps);
        let default_subapp = Arc::new(self.default_subapp);
        let error_handler = Arc::new(self.error_handler);

        self.thread_pool.register_monitor(self.monitor.clone());
        self.thread_pool.start();

        if self.force_https {
            let cloned_monitor = self.monitor.clone();

            if self.thread_pool.thread_count() < 2 {
                println!("Error: A minimum of two threads are required to force HTTPS since one is required for redirects.");
                std::process::exit(1);
            }

            self.thread_pool
                .execute(|| force_https_thread(cloned_monitor).unwrap_or(()));
        }

        for sock in socket.incoming() {
            match sock {
                Ok(mut sock) => {
                    let cloned_state = self.state.clone();

                    // Check that the client is allowed to connect
                    if (self.connection_condition)(&mut sock, cloned_state) {
                        let cloned_state = self.state.clone();
                        let cloned_subapps = subapps.clone();
                        let cloned_default_subapp = default_subapp.clone();
                        let cloned_error_handler = error_handler.clone();
                        let cloned_handler = self.connection_handler;
                        let cloned_timeout = self.connection_timeout;
                        let cloned_monitor = self.monitor.clone();
                        let cloned_config = self
                            .tls_config
                            .as_ref()
                            .expect("TLS certificate not supplied")
                            .clone();

                        cloned_monitor.send(
                            Event::new(EventType::ConnectionSuccess)
                                .with_peer_result(sock.peer_addr()),
                        );

                        // Spawn a new thread to handle the connection
                        self.thread_pool.execute(move || {
                            cloned_monitor.send(
                                Event::new(EventType::ThreadPoolProcessStarted)
                                    .with_peer_result(sock.peer_addr()),
                            );

                            let server = ServerConnection::new(cloned_config).unwrap();
                            let tls_stream = rustls::StreamOwned::new(server, sock);
                            let stream = Stream::Tls(tls_stream);

                            (cloned_handler)(
                                stream,
                                cloned_subapps,
                                cloned_default_subapp,
                                cloned_error_handler,
                                cloned_state,
                                cloned_monitor,
                                cloned_timeout,
                            )
                        });
                    } else {
                        self.monitor.send(
                            Event::new(EventType::ConnectionDenied)
                                .with_peer_result(sock.peer_addr()),
                        );
                    }
                }
                Err(e) => self
                    .monitor
                    .send(Event::new(EventType::ConnectionError).with_info(e.to_string())),
            }
        }

        Ok(())
    }

    pub fn with_shutdown(mut self, shutdown: Arc<AtomicBool>) -> Self {
        self.shutdown = Some(shutdown);
        self
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

    /// Sets the default sub-app for the server.
    /// This overrides all the routes added, as they will be replaced by the routes in the default sub-app.
    pub fn with_default_subapp(mut self, subapp: SubApp<State>) -> Self {
        self.default_subapp = subapp;
        self
    }

    /// Registers a monitor for the server.
    pub fn with_monitor(mut self, monitor: MonitorConfig) -> Self {
        self.monitor = monitor;
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

    /// Sets the connection timeout, the amount of time to wait between keep-alive requests.
    pub fn with_connection_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.connection_timeout = timeout;
        self
    }

    /// Sets the CORS configuration for the app.
    ///
    /// This overrides the CORS configuration for existing and future individual routes.
    ///
    /// To simply allow every origin, method and header, use `Cors::wildcard()`.
    pub fn with_cors(mut self, cors: Cors) -> Self {
        self.default_subapp = self.default_subapp.with_cors(cors);
        self
    }

    /// Sets the CORS configuration for the specified route.
    ///
    /// To simply allow every origin, method and header, use `Cors::wildcard()`.
    pub fn with_cors_config(mut self, route: &str, cors: Cors) -> Self {
        self.default_subapp = self.default_subapp.with_cors_config(route, cors);
        self
    }

    /// Sets whether HTTPS should be forced on all connections. Defaults to false.
    ///
    /// If this is set to true, a background thread will be spawned when `run_tls` is called to send
    ///   redirect responses to all insecure requests on port 80.
    #[cfg(feature = "tls")]
    pub fn with_forced_https(mut self, forced: bool) -> Self {
        self.force_https = forced;
        self
    }

    /// Sets the TLS configuration for the server.
    ///
    /// This **must** be called before `run_tls` is called.
    #[cfg(feature = "tls")]
    pub fn with_cert(mut self, cert_path: impl AsRef<str>, key_path: impl AsRef<str>) -> Self {
        use rustls::{Certificate, PrivateKey};
        use rustls_pemfile::{read_one, Item};

        use std::fs::File;
        use std::io::BufReader;

        let mut cert_file =
            BufReader::new(File::open(cert_path.as_ref()).expect("failed to open cert file"));
        let mut key_file =
            BufReader::new(File::open(key_path.as_ref()).expect("failed to open key file"));

        let certs: Vec<Certificate> = match read_one(&mut cert_file).unwrap().unwrap() {
            Item::X509Certificate(cert) => vec![Certificate(cert)],
            _ => panic!("failed to parse cert file"),
        };

        let key: PrivateKey = match read_one(&mut key_file).unwrap().unwrap() {
            Item::PKCS8Key(key) => PrivateKey(key),
            _ => panic!("failed to parse key file"),
        };

        let config = Arc::new(
            ServerConfig::builder()
                .with_safe_defaults()
                .with_no_client_auth()
                .with_single_cert(certs, key)
                .expect("failed to create server config"),
        );

        self.tls_config = Some(config);

        self
    }

    /// Adds a global WebSocket handler to the server.
    ///
    /// ## Deprecated
    /// This function is deprecated and will be removed in a future version.
    /// Please use `with_websocket_route` instead.
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
///   received without the `Connection: Keep-Alive` header.
#[allow(clippy::too_many_arguments)]
fn client_handler<State>(
    mut stream: Stream,
    subapps: Arc<Vec<SubApp<State>>>,
    default_subapp: Arc<SubApp<State>>,
    error_handler: Arc<ErrorHandler>,
    state: Arc<State>,
    monitor: MonitorConfig,
    timeout: Option<Duration>,
) {
    let addr = if let Ok(addr) = stream.peer_addr() {
        addr
    } else {
        monitor.send(EventType::StreamDisconnectedWhileWaiting);

        return;
    };

    loop {
        // Parses the request from the stream
        let request = match timeout {
            Some(timeout) => Request::from_stream_with_timeout(&mut stream, addr, timeout),
            None => Request::from_stream(&mut stream, addr),
        };

        let cloned_state = state.clone();

        // If the request is valid an is a WebSocket request, call the corresponding handler
        if let Ok(req) = &request {
            if req.headers.get(&HeaderType::Upgrade) == Some("websocket") {
                monitor.send(Event::new(EventType::WebsocketConnectionRequested).with_peer(addr));

                call_websocket_handler(req, &subapps, &default_subapp, cloned_state, stream);

                monitor.send(Event::new(EventType::WebsocketConnectionClosed).with_peer(addr));
                break;
            }
        }

        // Get the keep alive information from the request before it is consumed by the handler
        let keep_alive = if let Ok(request) = &request {
            if let Some(connection) = request.headers.get(&HeaderType::Connection) {
                connection.to_ascii_lowercase() == "keep-alive"
            } else {
                false
            }
        } else {
            false
        };

        // Generate the response based on the handlers
        let response = match &request {
            Ok(request) if request.method == Method::Options => {
                let handler = get_handler(request, &subapps, &default_subapp);

                match handler {
                    Some(handler) => {
                        let mut response = Response::empty(StatusCode::NoContent)
                            .with_header(HeaderType::Date, DateTime::now().to_string())
                            .with_header(HeaderType::Server, "Humphrey")
                            .with_header(
                                HeaderType::Connection,
                                match keep_alive {
                                    true => "Keep-Alive",
                                    false => "Close",
                                },
                            );

                        handler.cors.set_headers(&mut response.headers);

                        response
                    }
                    None => error_handler(StatusCode::NotFound),
                }
            }
            Ok(request) => {
                let handler = get_handler(request, &subapps, &default_subapp);

                let mut response = match handler {
                    Some(handler) => {
                        let mut response: Response =
                            handler.handler.serve(request.clone(), state.clone());

                        handler.cors.set_headers(&mut response.headers);

                        response
                    }
                    None => error_handler(StatusCode::NotFound),
                };

                // Automatically generate required headers
                match response.headers.get_mut(HeaderType::Connection) {
                    Some(_) => (),
                    None => {
                        if let Some(connection) = &request.headers.get(&HeaderType::Connection) {
                            response.headers.add(HeaderType::Connection, connection);
                        } else {
                            response.headers.add(HeaderType::Connection, "Close");
                        }
                    }
                }

                match response.headers.get_mut(HeaderType::Server) {
                    Some(_) => (),
                    None => {
                        response.headers.add(HeaderType::Server, "Humphrey");
                    }
                }

                match response.headers.get_mut(HeaderType::Date) {
                    Some(_) => (),
                    None => {
                        response
                            .headers
                            .add(HeaderType::Date, DateTime::now().to_string());
                    }
                }

                match response.headers.get_mut(HeaderType::ContentLength) {
                    Some(_) => (),
                    None => {
                        response
                            .headers
                            .add(HeaderType::ContentLength, response.body.len().to_string());
                    }
                }

                // Set HTTP version
                response.version = request.version.clone();

                response
            }
            Err(e) => match e {
                RequestError::Request => error_handler(StatusCode::BadRequest),
                RequestError::Timeout => error_handler(StatusCode::RequestTimeout),
                RequestError::Disconnected => return,
                RequestError::Stream => {
                    return monitor.send(Event::new(EventType::RequestServedError))
                }
            },
        };

        // Write the response to the stream
        let status = response.status_code;
        let response_bytes: Vec<u8> = response.into();

        if let Err(e) = stream.write_all(&response_bytes) {
            monitor.send(
                Event::new(EventType::RequestServedError)
                    .with_peer(addr)
                    .with_info(e.to_string()),
            );

            break;
        };

        let status_str: &str = status.into();

        match status {
            StatusCode::OK => monitor.send(
                Event::new(EventType::RequestServedSuccess)
                    .with_peer(addr)
                    .with_info(format!("200 OK {}", request.unwrap().uri)),
            ),
            StatusCode::RequestTimeout => monitor.send(
                Event::new(EventType::RequestTimeout)
                    .with_peer(addr)
                    .with_info("408 Request Timeout"),
            ),
            e => {
                if let Ok(request) = request {
                    monitor.send(
                        Event::new(EventType::RequestServedError)
                            .with_peer(addr)
                            .with_info(format!("{} {} {}", u16::from(e), status_str, request.uri)),
                    )
                } else {
                    monitor.send(
                        Event::new(EventType::RequestServedError)
                            .with_peer(addr)
                            .with_info(format!("{} {}", u16::from(e), status_str)),
                    )
                }
            }
        }

        // If the request specified to keep the connection open, respect this
        if !keep_alive {
            break;
        }

        monitor.send(Event::new(EventType::KeepAliveRespected).with_peer(addr));
    }

    monitor.send(Event::new(EventType::ConnectionClosed).with_peer(addr));
}

/// Gets the correct handler for the given request.
pub(crate) fn get_handler<'a, State>(
    request: &'a Request,
    subapps: &'a [SubApp<State>],
    default_subapp: &'a SubApp<State>,
) -> Option<&'a RouteHandler<State>> {
    // Iterate over the sub-apps and find the one which matches the host
    if let Some(host) = request.headers.get(&HeaderType::Host) {
        if let Some(subapp) = subapps
            .iter()
            .find(|subapp| wildcard_match(&subapp.host, host))
        {
            // If the sub-app has a handler for this route, call it
            if let Some(handler) = subapp
                .routes // Get the routes of the sub-app
                .iter() // Iterate over the routes
                .find(|route| route.route.route_matches(&request.uri))
            // Find the route that matches
            {
                return Some(handler);
            }
        }
    }

    // If no sub-app was found, try to use the handler on the default sub-app
    if let Some(handler) = default_subapp
        .routes
        .iter()
        .find(|route| route.route.route_matches(&request.uri))
    {
        return Some(handler);
    }

    None
}

/// Calls the correct WebSocket handler for the given request.
fn call_websocket_handler<State>(
    request: &Request,
    subapps: &[SubApp<State>],
    default_subapp: &SubApp<State>,
    state: Arc<State>,
    stream: Stream,
) {
    // Iterate over the sub-apps and find the one which matches the host
    if let Some(host) = request.headers.get(&HeaderType::Host) {
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
                handler.handler.serve(request.clone(), stream, state);
                return;
            }
        }
    }

    // If no sub-app was found, try to use the handler on the default sub-app
    if let Some(handler) = default_subapp
        .websocket_routes
        .iter()
        .find(|route| route.route.route_matches(&request.uri))
    {
        handler.handler.serve(request.clone(), stream, state)
    }
}

#[cfg(feature = "tls")]
fn force_https_thread(monitor: MonitorConfig) -> Result<(), Box<dyn std::error::Error>> {
    let socket = TcpListener::bind("0.0.0.0:80")?;

    for mut stream in socket.incoming().flatten() {
        let addr = stream.peer_addr()?;
        let request = Request::from_stream(&mut stream, addr)?;

        let response = if let Some(host) = request.headers.get(&HeaderType::Host) {
            Response::empty(StatusCode::MovedPermanently)
                .with_header(
                    HeaderType::Location,
                    format!("https://{}{}", host, request.uri),
                )
                .with_header(HeaderType::Connection, "Close")
        } else {
            Response::empty(StatusCode::OK)
                .with_bytes(b"<h1>Please access over HTTPS</h1>")
                .with_header(HeaderType::ContentLength, "33")
                .with_header(HeaderType::Connection, "Close")
        };

        let response_bytes: Vec<u8> = response.into();
        stream.write_all(&response_bytes)?;

        monitor.send(Event::new(EventType::HTTPSRedirect).with_peer(addr));
    }

    Ok(())
}

/// The default error handler for every Humphrey app.
/// This can be overridden by using the `with_error_handler` method when building the app.
pub(crate) fn error_handler(status_code: StatusCode) -> Response {
    let body = format!(
        "<html><body><h1>{} {}</h1></body></html>",
        Into::<u16>::into(status_code),
        Into::<&str>::into(status_code)
    );

    Response::new(status_code, body.as_bytes())
}

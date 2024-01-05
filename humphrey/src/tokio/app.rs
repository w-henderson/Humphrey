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

use std::sync::Arc;

use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio_util::sync::CancellationToken;

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
    subapps: Vec<SubApp<State>>,
    default_subapp: SubApp<State>,
    error_handler: ErrorHandler,
    state: Arc<State>,
    monitor: MonitorConfig,
    connection_condition: ConnectionCondition<State>,
    #[cfg(feature = "tls")]
    tls_config: Option<Arc<ServerConfig>>,
    #[cfg(feature = "tls")]
    force_https: bool,
    shutdown: Option<CancellationToken>,
}

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
            subapps: Vec::new(),
            default_subapp: SubApp::default(),
            error_handler,
            state: Arc::new(State::default()),
            monitor: MonitorConfig::default(),
            connection_condition: |_, _| true,
            #[cfg(feature = "tls")]
            tls_config: None,
            #[cfg(feature = "tls")]
            force_https: false,
            shutdown: None,
        }
    }

    /// Initialises a new Humphrey app with the given configuration options.
    pub fn new_with_config(state: State) -> Self {
        Self {
            subapps: Vec::new(),
            default_subapp: SubApp::default(),
            error_handler,
            state: Arc::new(state),
            monitor: MonitorConfig::default(),
            connection_condition: |_, _| true,
            #[cfg(feature = "tls")]
            tls_config: None,
            #[cfg(feature = "tls")]
            force_https: false,
            shutdown: None,
        }
    }

    /// Runs the Humphrey app on the given socket address.
    /// This function will only return if a fatal error is thrown such as the port being in use.
    pub async fn run<A>(self, addr: A) -> Result<(), HumphreyError>
    where
        A: ToSocketAddrs,
    {
        let socket = TcpListener::bind(addr).await?;
        let subapps = Arc::new(self.subapps);
        let default_subapp = Arc::new(self.default_subapp);
        let error_handler = Arc::new(self.error_handler);

        loop {
            let shutdown = async {
                if let Some(sd) = self.shutdown.as_ref() {
                    sd.cancelled().await
                } else {
                    futures::future::pending().await
                }
            };
            tokio::select! {
                () = shutdown => { break Ok(()); }
                s = socket.accept() => {
                    match s {
                        Ok((mut stream, _)) => {
                            let cloned_state = self.state.clone();

                            // Check that the client is allowed to connect
                            if (self.connection_condition)(&mut stream, cloned_state) {
                                let cloned_state = self.state.clone();
                                let cloned_monitor = self.monitor.clone();
                                let cloned_subapps = subapps.clone();
                                let cloned_default_subapp = default_subapp.clone();
                                let cloned_error_handler = error_handler.clone();

                                cloned_monitor.send(
                                    Event::new(EventType::ConnectionSuccess)
                                        .with_peer_result(stream.peer_addr()),
                                );

                                // Spawn a new thread to handle the connection
                                tokio::spawn(async move {
                                    cloned_monitor.send(
                                        Event::new(EventType::ThreadPoolProcessStarted)
                                            .with_peer_result(stream.peer_addr()),
                                    );

                                    client_handler(
                                        Stream::Tcp(stream),
                                        cloned_subapps,
                                        cloned_default_subapp,
                                        cloned_error_handler,
                                        cloned_state,
                                        cloned_monitor,
                                    )
                                        .await
                                });
                            } else {
                                self.monitor.send(
                                    Event::new(EventType::ConnectionDenied)
                                        .with_peer_result(stream.peer_addr()),
                                );
                            }
                        }
                        Err(e) => self
                            .monitor
                            .send(Event::new(EventType::ConnectionError).with_info(e.to_string())),
                    }
                }
            };
        }
    }

    /// Securely runs the Humphrey app on the given socket address.
    /// This function will only return if a fatal error is thrown such as the port being in use or the TLS certificate being invalid.
    #[cfg(feature = "tls")]
    pub async fn run_tls<A>(self, addr: A) -> Result<(), HumphreyError>
    where
        A: ToSocketAddrs,
    {
        use tokio_rustls::TlsAcceptor;

        let socket = TcpListener::bind(addr).await?;
        let subapps = Arc::new(self.subapps);
        let default_subapp = Arc::new(self.default_subapp);
        let error_handler = Arc::new(self.error_handler);
        let tls_config = self.tls_config.expect("TLS certificate not supplied");

        if self.force_https {
            let cloned_monitor = self.monitor.clone();

            tokio::spawn(async move {
                force_https_thread(cloned_monitor).await.unwrap_or(());
            });
        }

        let acceptor = TlsAcceptor::from(tls_config);

        loop {
            let shutdown = async {
                if let Some(sd) = self.shutdown.as_ref() {
                    sd.cancelled().await
                } else {
                    futures::future::pending().await
                }
            };

            tokio::select! {
                () = shutdown => { break Ok(()); }
                 s = socket.accept() => {
                    match s {
                        Ok((mut sock, _)) => {
                            let cloned_state = self.state.clone();

                            // Check that the client is allowed to connect
                            if (self.connection_condition)(&mut sock, cloned_state) {
                                let cloned_state = self.state.clone();
                                let cloned_subapps = subapps.clone();
                                let cloned_default_subapp = default_subapp.clone();
                                let cloned_error_handler = error_handler.clone();
                                let cloned_monitor = self.monitor.clone();
                                let cloned_acceptor = acceptor.clone();

                                cloned_monitor.send(
                                    Event::new(EventType::ConnectionSuccess)
                                        .with_peer_result(sock.peer_addr()),
                                );

                                // Spawn a new thread to handle the connection
                                tokio::spawn(async move {
                                    cloned_monitor.send(
                                        Event::new(EventType::ThreadPoolProcessStarted)
                                            .with_peer_result(sock.peer_addr()),
                                    );

                                    match cloned_acceptor.accept(sock).await {
                                        Ok(tls_stream) => {
                                            let stream = Stream::Tls(tls_stream);

                                            client_handler(
                                                stream,
                                                cloned_subapps,
                                                cloned_default_subapp,
                                                cloned_error_handler,
                                                cloned_state,
                                                cloned_monitor,
                                            )
                                                .await
                                        }
                                        Err(e) => cloned_monitor.send(
                                            Event::new(EventType::ConnectionError).with_info(e.to_string()),
                                        ),
                                    }
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
            }
        }
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

    /// Registers a shutdown signal to gracefully shutdown the app, ending the run/run_tls loop.
    pub fn with_shutdown(mut self, cancel_token: CancellationToken) -> Self {
        self.shutdown = Some(cancel_token);
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
async fn client_handler<State>(
    mut stream: Stream,
    subapps: Arc<Vec<SubApp<State>>>,
    default_subapp: Arc<SubApp<State>>,
    error_handler: Arc<ErrorHandler>,
    state: Arc<State>,
    monitor: MonitorConfig,
) {
    let addr = if let Ok(addr) = stream.peer_addr() {
        addr
    } else {
        monitor.send(EventType::StreamDisconnectedWhileWaiting);

        return;
    };

    loop {
        // Parses the request from the stream
        let request = Request::from_stream(&mut stream, addr).await;

        let cloned_state = state.clone();

        // If the request is valid an is a WebSocket request, call the corresponding handler
        if let Ok(req) = &request {
            if req.headers.get(&HeaderType::Upgrade) == Some("websocket") {
                monitor.send(Event::new(EventType::WebsocketConnectionRequested).with_peer(addr));

                call_websocket_handler(req, &subapps, &default_subapp, cloned_state, stream).await;

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
                            handler.handler.serve(request.clone(), state.clone()).await;

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

        if let Err(e) = stream.write_all(&response_bytes).await {
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
async fn call_websocket_handler<State>(
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
                handler.handler.serve(request.clone(), stream, state).await;
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
        handler.handler.serve(request.clone(), stream, state).await
    }
}

#[cfg(feature = "tls")]
async fn force_https_thread(monitor: MonitorConfig) -> Result<(), Box<dyn std::error::Error>> {
    let socket = TcpListener::bind("0.0.0.0:80").await?;

    while let Ok((mut stream, addr)) = socket.accept().await {
        let request = Request::from_stream(&mut stream, addr).await?;

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
        stream.write_all(&response_bytes).await?;

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

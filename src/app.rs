use crate::http::headers::RequestHeader;
use crate::http::request::{Request, RequestError};
use crate::http::response::Response;
use crate::http::status::StatusCode;
use crate::route::RouteHandler;

use std::io::Write;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::spawn;

pub struct App<State>
where
    State: Send + Default + 'static,
{
    routes: Vec<RouteHandler<State>>,
    error_handler: ErrorHandler,
    state: Arc<Mutex<State>>,
}

pub type RequestHandler<State> = fn(&Request, Arc<Mutex<State>>) -> Response;
pub type ErrorHandler = fn(Option<&Request>, StatusCode) -> Response;
pub type HumphreyError = Box<dyn std::error::Error>;

impl<State> App<State>
where
    State: Send + Default + 'static,
{
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            error_handler,
            state: Arc::new(Mutex::new(State::default())),
        }
    }

    pub fn run(self, addr: &SocketAddr) -> Result<(), HumphreyError> {
        let socket = TcpListener::bind(addr)?;
        let routes = Arc::new(self.routes);
        let error_handler = Arc::new(self.error_handler);

        for stream in socket.incoming() {
            match stream {
                Ok(stream) => {
                    let cloned_routes = routes.clone();
                    let cloned_error_handler = error_handler.clone();
                    let cloned_state = self.state.clone();
                    spawn(move || {
                        client_handler(stream, cloned_routes, cloned_error_handler, cloned_state)
                    });
                }
                Err(_) => (),
            }
        }

        Ok(())
    }

    pub fn with_route(mut self, route: &str, handler: RequestHandler<State>) -> Self {
        self.routes.push(RouteHandler {
            route: route.parse().unwrap(),
            handler,
        });
        self
    }

    pub fn with_error_handler(mut self, handler: ErrorHandler) -> Self {
        self.error_handler = handler;
        self
    }
}

fn client_handler<State>(
    mut stream: TcpStream,
    routes: Arc<Vec<RouteHandler<State>>>,
    error_handler: Arc<ErrorHandler>,
    state: Arc<Mutex<State>>,
) {
    loop {
        let request = Request::from_stream(&mut stream);
        let cloned_state = state.clone();

        if match &request {
            Ok(_) => false,
            Err(e) => e == &RequestError::Stream,
        } {
            break;
        }

        let response = match &request {
            Ok(request) => match routes.iter().find(|r| r.route.matches(&request.uri)) {
                Some(handler) => (handler.handler)(request, cloned_state),
                None => error_handler(Some(request), StatusCode::NotFound),
            },
            Err(_) => error_handler(None, StatusCode::BadRequest),
        };

        let response_bytes: Vec<u8> = response.into();
        if let Err(_) = stream.write(&response_bytes) {
            break;
        };

        if let Ok(request) = request {
            if let Some(connection) = request.headers.get(&RequestHeader::Connection) {
                if connection.to_ascii_lowercase() != "keep-alive" {
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }
}

fn error_handler(request: Option<&Request>, status_code: StatusCode) -> Response {
    let body = format!(
        "<html><body><h1>{} {}</h1></body></html>",
        Into::<u16>::into(status_code.clone()),
        Into::<&str>::into(status_code.clone())
    );

    if let Some(request) = request {
        Response::new(status_code)
            .with_bytes(body.as_bytes().to_vec())
            .with_request_compatibility(request)
            .with_generated_headers()
    } else {
        Response::new(status_code)
            .with_bytes(body.as_bytes().to_vec())
            .with_generated_headers()
    }
}

use crate::app::{call_handler, ErrorHandler};
use crate::http::date::DateTime;
use crate::http::headers::{RequestHeader, ResponseHeader};
use crate::http::request::RequestError;
use crate::http::{Request, StatusCode};
use crate::krauss::wildcard_match;
use crate::route::{Route, SubApp};

use std::collections::btree_map::Entry;
use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use rustls::{ServerConfig, ServerConnection, Stream};

/// Handles a connection with a client.
/// The connection will be opened upon the first request and closed as soon as a request is
///   recieved without the `Connection: Keep-Alive` header.
pub fn client_handler<State>(
    mut stream: TcpStream,
    subapps: Arc<Vec<SubApp<State>>>,
    default_subapp: Arc<SubApp<State>>,
    error_handler: Arc<ErrorHandler>,
    state: Arc<State>,
    config: Arc<ServerConfig>,
) {
    let addr = stream.peer_addr().unwrap();

    let mut server = ServerConnection::new(config).unwrap();
    let mut tls_stream = Stream::new(&mut server, &mut stream);

    loop {
        // Parses the request from the stream
        let request = Request::from_stream(&mut tls_stream, addr);
        let cloned_state = state.clone();

        // If the request is valid an is a WebSocket request, call the corresponding handler
        if let Ok(req) = &request {
            if req.headers.get(&RequestHeader::Upgrade) == Some(&"websocket".to_string()) {
                call_websocket_handler(req, &subapps, &default_subapp, cloned_state, tls_stream);
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
        if tls_stream.write(&response_bytes).is_err() {
            break;
        };

        // If the request specified to keep the connection open, respect this
        if !keep_alive {
            break;
        }
    }
}

/// Calls the correct WebSocket handler for the given request.
fn call_websocket_handler<State>(
    request: &Request,
    subapps: &[SubApp<State>],
    default_subapp: &SubApp<State>,
    state: Arc<State>,
    stream: Stream<ServerConnection, TcpStream>,
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

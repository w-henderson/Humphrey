use crate::error::WebsocketError;
use crate::stream::WebsocketStream;
use crate::util::base64::Base64Encode;
use crate::util::sha1::SHA1Hash;
use crate::MAGIC_STRING;

use humphrey::http::headers::{RequestHeader, ResponseHeader};
use humphrey::http::{Request, Response, StatusCode};

use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

pub trait WebsocketHandler<S>: Fn(WebsocketStream<TcpStream>, Arc<S>) + Send + Sync {}
impl<T, S> WebsocketHandler<S> for T where T: Fn(WebsocketStream<TcpStream>, Arc<S>) + Send + Sync {}

pub fn websocket_handler<T, S>(handler: T) -> impl Fn(Request, TcpStream, Arc<S>)
where
    T: WebsocketHandler<S>,
{
    move |request: Request, mut stream: TcpStream, state: Arc<S>| {
        if handshake(request, &mut stream).is_ok() {
            handler(WebsocketStream::new(stream), state);
        }
    }
}

fn handshake(request: Request, stream: &mut TcpStream) -> Result<(), WebsocketError> {
    // Get the handshake key header
    let handshake_key = request
        .headers
        .get(&RequestHeader::Custom {
            name: "sec-websocket-key".into(),
        })
        .ok_or(WebsocketError::HandshakeError)?;

    // Calculate the handshake response
    let sec_websocket_accept = format!("{}{}", handshake_key, MAGIC_STRING).hash().encode();

    // Serialise the handshake response
    let response = Response::empty(StatusCode::SwitchingProtocols)
        .with_header(ResponseHeader::Upgrade, "websocket".into())
        .with_header(ResponseHeader::Connection, "Upgrade".into())
        .with_header(
            ResponseHeader::Custom {
                name: "Sec-WebSocket-Accept".into(),
            },
            sec_websocket_accept,
        );

    // Transmit the handshake response
    let response_bytes: Vec<u8> = response.into();
    stream
        .write_all(&response_bytes)
        .map_err(|_| WebsocketError::WriteError)?;

    Ok(())
}

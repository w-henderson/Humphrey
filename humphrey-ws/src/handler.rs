//! Provides a Humphrey-compatible WebSocket handler for performing the handshake.

use crate::error::WebsocketError;
use crate::stream::WebsocketStream;
use crate::util::base64::Base64Encode;
use crate::util::sha1::SHA1Hash;
use crate::MAGIC_STRING;

use humphrey::http::headers::{RequestHeader, ResponseHeader};
use humphrey::http::{Request, Response, StatusCode};
use humphrey::stream::Stream;

use std::io::Write;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

/// Represents a function able to handle WebSocket streams.
pub trait WebsocketHandler<S>: Fn(WebsocketStream<Stream>, Arc<S>) + Send + Sync {}
impl<T, S> WebsocketHandler<S> for T where T: Fn(WebsocketStream<Stream>, Arc<S>) + Send + Sync {}

/// Provides WebSocket handshake functionality.
/// Supply a `WebsocketHandler` to handle the subsequent messages.
///
/// ## Example
/// ```
/// use humphrey::App;
/// use humphrey::stream::Stream;
/// use humphrey_ws::message::Message;
/// use humphrey_ws::stream::WebsocketStream;
/// use humphrey_ws::websocket_handler;
///
/// use std::sync::Arc;
///
/// fn main() {
///     let app: App<()> = App::new()
///         .with_websocket_route("/", websocket_handler(my_handler));
///
///     app.run("0.0.0.0:80").unwrap();
/// }
///
/// fn my_handler(mut stream: WebsocketStream<Stream>, _: Arc<()>) {
///     stream.send(Message::new("Hello, World!")).unwrap();
/// }
/// ```
pub fn websocket_handler<T, S>(handler: T) -> impl Fn(Request, Stream, Arc<S>)
where
    T: WebsocketHandler<S>,
{
    move |request: Request, mut stream: Stream, state: Arc<S>| {
        if handshake(request, &mut stream).is_ok() {
            handler(WebsocketStream::new(stream), state);
        }
    }
}

/// Provides asynchronous WebSocket functionality.
/// Supply a hook to an asynchronous WebSocket app to handle the subsequent messages.
///
/// ## Example
/// ```
/// use humphrey::App;
/// use humphrey_ws::async_app::{AsyncStream, AsyncWebsocketApp};
/// use humphrey_ws::handler::async_websocket_handler;
/// use humphrey_ws::message::Message;
///
/// use std::sync::Arc;
/// use std::thread::spawn;
///
/// fn main() {
///     let websocket_app: AsyncWebsocketApp<()> =
///         AsyncWebsocketApp::new().with_message_handler(message_handler);
///
///     let humphrey_app: App<()> = App::new()
///         .with_websocket_route("/ws", async_websocket_handler(websocket_app.connect_hook()));
///
///     spawn(move || humphrey_app.run("0.0.0.0:80").unwrap());
///
///     websocket_app.run();
/// }
///
/// fn message_handler(stream: AsyncStream, message: Message, _: Arc<()>) {
///     println!(
///         "{}: Message received: {}",
///         stream.peer_addr(),
///         message.text().unwrap().trim()
///     );
///
///     stream.send(Message::new("Message received!"));
/// }
/// ```
pub fn async_websocket_handler<S>(
    hook: Arc<Mutex<Sender<WebsocketStream<Stream>>>>,
) -> impl Fn(Request, Stream, Arc<S>) {
    move |request: Request, mut stream: Stream, _: Arc<S>| {
        if handshake(request, &mut stream).is_ok() {
            hook.lock()
                .unwrap()
                .send(WebsocketStream::new(stream))
                .unwrap();
        }
    }
}

/// Performs the WebSocket handshake.
fn handshake(request: Request, stream: &mut Stream) -> Result<(), WebsocketError> {
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

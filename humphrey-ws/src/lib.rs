//! Humphrey WebSocket is a crate which extends Humphrey Core with WebSocket support by hooking into the latter's `WebsocketHandler` trait. It handles the WebSocket handshake and framing protocol and provides a simple and flexible API for sending and receiving messages. Using Humphrey's generic `Stream` type, it supports drop-in TLS. It also has no dependencies in accordance with Humphrey's goals of being dependency-free.
//!
//! It provides both synchronous and asynchronous WebSocket functionality.
//!
//! Learn more about Humphrey WebSocket [here](https://humphrey.whenderson.dev/websocket/index.html).

#![warn(missing_docs)]

const MAGIC_STRING: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub mod async_app;
pub mod error;
pub mod handler;
pub mod message;
pub mod ping;
pub mod stream;

pub use handler::async_websocket_handler;
pub use handler::websocket_handler;

pub use async_app::{AsyncStream, AsyncWebsocketApp};
pub use message::Message;
pub use stream::WebsocketStream;

pub use util::restion;

mod frame;
mod util;

#[cfg(test)]
mod tests;

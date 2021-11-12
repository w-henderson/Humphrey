const MAGIC_STRING: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub mod error;
pub mod handler;
pub mod message;
pub mod stream;

pub use handler::websocket_handler;

mod frame;
mod util;

#[cfg(test)]
mod tests;

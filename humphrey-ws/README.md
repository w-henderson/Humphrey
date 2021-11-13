<p align="center">
  <img src="https://raw.githubusercontent.com/w-henderson/Humphrey/master/assets/logo.png" width=250><br><br>
  <img src="https://img.shields.io/badge/language-rust-b07858?style=for-the-badge&logo=rust" style="margin-right:5px">
  <img src="https://img.shields.io/github/workflow/status/w-henderson/Humphrey/CI?style=for-the-badge" style="margin-right:5px">
  <img src="https://img.shields.io/crates/v/humphrey-ws?style=for-the-badge" style="margin-right:5px">
</p>

# WebSocket support for Humphrey.
The core Humphrey crate does not support WebSocket connections, but through its `WebsocketHandler` trait, it can be extended to support them, which is what this crate does.

## Features
- Performs WebSocket handshake and implements WebSocket according to [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455).
- Well-integrated with the core Humphrey crate
- Has no dependencies in accordance with Humphrey's goals of being dependency-free. This means SHA-1 ([RFC 3174](https://datatracker.ietf.org/doc/html/rfc3174)) and base64 ([RFC 4648](https://datatracker.ietf.org/doc/html/rfc4648)) are both implemented from scratch in this crate as they are required for the handshake.

## Installation
The Humphrey WebSocket crate can be installed by adding `humphrey_ws` to your dependencies in your `Cargo.toml` file.

## Documentation
The Humphrey WebSocket documentation can be found at [docs.rs](https://docs.rs/humphrey-ws/).

## Basic Example
```rs
use humphrey::App;

use humphrey_ws::error::WebsocketError;
use humphrey_ws::message::Message;
use humphrey_ws::stream::WebsocketStream;
use humphrey_ws::websocket_handler;

use std::net::TcpStream;
use std::sync::Arc;

fn main() {
    let app: App<()> = App::new()
        .with_websocket_handler(websocket_handler(my_handler));
    app.run("0.0.0.0:80").unwrap();
}

fn my_handler(mut stream: WebsocketStream<TcpStream>, _: Arc<()>) {
    let hello_world = Message::new("Hello, World!");
    stream.send(hello_world).unwrap();

    loop {
        match stream.recv() {
            Ok(msg) => println!("Message: {}", msg.text().unwrap()),
            Err(WebsocketError::ConnectionClosed) => {
                break;
            },
            _ => ()
        }
    }

    println!("Connection closed");
}
```

## Further Examples
- [Echo Server](https://github.com/w-henderson/Humphrey/tree/master/examples/websocket): echoes received messages back to the client with an incrementing number at the end
use humphrey::{handlers, App};

use humphrey_ws::error::WebsocketError;
use humphrey_ws::message::Message;
use humphrey_ws::stream::WebsocketStream;
use humphrey_ws::websocket_handler;

use std::error::Error;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// App state with a simple counter, as explained in the `stateful` example.
#[derive(Default)]
struct AppState {
    counter: AtomicUsize,
}

fn main() -> Result<(), Box<dyn Error>> {
    let app: App<AppState> = App::new()
        // Serve the `static` directory to regular HTTP requests.
        .with_path_aware_route("/*", handlers::serve_dir("./static"))
        // Use the `humphrey_ws` WebSocket handler to wrap our own echo handler.
        .with_websocket_route("/ws", websocket_handler(echo_handler));
    app.run("0.0.0.0:80")?;

    Ok(())
}

/// Handler for WebSocket connections.
/// This is wrapped in `websocket_handler` to manage the handshake for us using the `humphrey_ws` crate.
fn echo_handler(mut stream: WebsocketStream, state: Arc<AppState>) {
    // Get the address of the client.
    let addr = stream.inner().peer_addr().unwrap().ip().to_string();

    println!("New connection from {}", addr);

    // Loop while the client is connected.
    loop {
        // Block while waiting to receive a message.
        match stream.recv() {
            // If the message was received successfully, echo it back with an increasing number at the end.
            Ok(message) => {
                let message = message.text().unwrap();
                let count = state.counter.fetch_add(1, Ordering::SeqCst);
                let response = format!("{} {}", message, count);

                // Send the WebSocket response
                stream.send(Message::new(response)).unwrap();

                println!(
                    "Received message `{}` from {}, echoing with the number {}",
                    message, addr, count
                )
            }
            // If the connection was closed, break out of the loop and clean up
            Err(WebsocketError::ConnectionClosed) => {
                break;
            }
            // Ignore any other errors
            _ => (),
        }
    }

    println!("Connection closed by {}", addr);
}

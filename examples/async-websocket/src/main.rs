use humphrey_ws::async_app::{AsyncSender, AsyncStream, AsyncWebsocketApp};
use humphrey_ws::message::Message;

use std::io::BufRead;
use std::sync::Arc;
use std::thread::spawn;

fn main() {
    // Create a new async WebSocket app with no state, and register some handlers.
    let websocket_app: AsyncWebsocketApp<()> = AsyncWebsocketApp::new()
        .with_connect_handler(connect_handler)
        .with_message_handler(message_handler);

    // Get a sender from the app so we can send messages without waiting for events.
    // Start a thread to listen for user input.
    let sender = websocket_app.sender();
    spawn(move || user_input(sender));

    // Run the app.
    websocket_app.run();
}

/// Listen for user input and broadcast it line by line to all connected clients.
fn user_input(sender: AsyncSender) {
    let stdin = std::io::stdin();
    let handle = stdin.lock();

    for line in handle.lines().flatten() {
        sender.broadcast(Message::new(line));
    }
}

/// Handle connections by broadcasting their arrival.
fn connect_handler(stream: AsyncStream, _: Arc<()>) {
    let message = Message::new(format!("Welcome, {}!", stream.peer_addr()));
    stream.broadcast(message);
}

/// Echo messages back to the client that sent them.
fn message_handler(stream: AsyncStream, message: Message, _: Arc<()>) {
    stream.send(message);
}

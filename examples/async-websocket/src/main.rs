use humphrey_ws::async_app::{AsyncSender, AsyncStream, AsyncWebsocketApp};
use humphrey_ws::message::Message;
use humphrey_ws::ping::Heartbeat;

use std::io::BufRead;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::spawn;

#[derive(Default)]
struct StreamState {
    messages_received: AtomicUsize,
}

fn main() {
    // Create a new async WebSocket app with no global state and a specified stream state, and register some handlers.
    let websocket_app: AsyncWebsocketApp<(), StreamState> = AsyncWebsocketApp::new()
        .with_heartbeat(Heartbeat::default())
        .with_connect_handler(connect_handler)
        .with_disconnect_handler(disconnect_handler)
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
fn connect_handler(stream: AsyncStream<StreamState>, _: Arc<()>) {
    let text = format!("Welcome, {}!", stream.peer_addr());
    let message = Message::new(text.clone());
    stream.broadcast(message);

    println!("{}", text);
}

/// Handle disconnections by broadcasting their departure.
fn disconnect_handler(stream: AsyncStream<StreamState>, _: Arc<()>) {
    let text = format!("{} has disconnected.", stream.peer_addr());
    let message = Message::new(text.clone());
    stream.broadcast(message);

    println!("{}", text);
}

/// Echo messages back to the client that sent them, along with their message number.
fn message_handler(stream: AsyncStream<StreamState>, message: Message, _: Arc<()>) {
    let message_number = stream
        .state
        .messages_received
        .fetch_add(1, Ordering::SeqCst)
        + 1;

    let text = format!(
        "{} (message number {})",
        message.text().unwrap(),
        message_number
    );

    stream.send(Message::new(text));
}

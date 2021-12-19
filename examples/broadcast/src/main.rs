use humphrey::{handlers, App};

use humphrey_ws::error::WebsocketError;
use humphrey_ws::message::Message;
use humphrey_ws::restion::Restion;
use humphrey_ws::stream::WebsocketStream;
use humphrey_ws::websocket_handler;

// We use the bus crate for a single-producer, multiple-consumer channel.
// We're actually using a fork of the crate to allow us to create new readers from different threads.
// For some reason, it hasn't been merged into the original crate yet.
//
// Thank you Adam Gausmann for the fork.
// https://github.com/agausmann/bus/tree/read_handle/lock
use bus::{Bus, BusReadHandle};

use std::error::Error;
use std::io::BufRead;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};
use std::time::Duration;

// For this example, the app state is simply a thread-safe read handle to the bus.
type AppState = Mutex<BusReadHandle<String>>;

fn main() -> Result<(), Box<dyn Error>> {
    // Initialise the bus and read handle.
    let bus: Bus<String> = Bus::new(16);
    let read_handle = bus.read_handle();

    // Move the bus onto its own thread to handle user input to broadcast.
    spawn(move || main_thread(bus));

    // Create a new app, specifying 32 threads and supplying the read handle as state.
    let app: App<AppState> = App::new_with_config(32, Mutex::new(read_handle))
        // Add a path-aware route to serve the static directory.
        .with_path_aware_route("/*", handlers::serve_dir("./static"))
        // Add a WebSocket handler to handle connections.
        .with_websocket_route("/ws", websocket_handler(broadcast_handler));
    app.run("0.0.0.0:80")?;

    Ok(())
}

// This function is run on a separate thread to handle user input to broadcast.
// It simply reads from the standard input and broadcasts every line to all connected clients.
fn main_thread(mut bus: Bus<String>) {
    let stdin = std::io::stdin();
    let handle = stdin.lock();

    println!("Type a message and it will be broadcasted to every connected client.");

    for line in handle.lines() {
        bus.broadcast(line.unwrap());
    }
}

// Handle WebSocket connections.
fn broadcast_handler(mut stream: WebsocketStream<TcpStream>, read_handle: Arc<AppState>) {
    // Get the address of the client.
    let addr = stream.inner().peer_addr().unwrap().ip().to_string();

    // Create a new reader for the bus.
    let mut rx = { read_handle.lock().unwrap().add_rx() };

    println!("New connection from {}", addr);

    loop {
        // Attempt to read a message from the WebSocket stream.
        match stream.recv_nonblocking() {
            // If successful, acknowledge the message.
            Restion::Ok(_) => {
                stream.send(Message::new("message acknowledged")).unwrap();
            }
            // If the connection was closed, break from the loop.
            Restion::Err(WebsocketError::ConnectionClosed) => {
                break;
            }
            // If otherwise, do nothing.
            // This includes when there are no messages to read.
            _ => (),
        }

        // Attempt to read a message from the bus.
        // If successful, send it to the WebSocket stream.
        if let Ok(message) = rx.try_recv() {
            stream
                .send(Message::new(format!("broadcast: {}", message)))
                .unwrap();
        }

        // Sleep a short while to avoid busy-waiting.
        sleep(Duration::from_millis(64));
    }

    println!("Connection closed by {}", addr);
}

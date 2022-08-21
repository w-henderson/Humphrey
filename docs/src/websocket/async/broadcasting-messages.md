# Broadcasting Messages
Many WebSocket applications broadcast messages to many clients at once, so in this chapter we'll learn how to do this asynchronously. Previously, we had to use an external dependency `bus`, but using the asynchronous approach, this is no longer necessary.

The example we build in this chapter will simply echo messages back to the client as well as broadcasting any messages typed into the server console to all connected clients. Furthermore, we'll broadcast a message whenever a client connects too.

## Initialising the Project
As before, we need a new Humphrey WebSocket application. We don't need to handle the disconnection event, so we won't add a handler for it.

```rs
use humphrey_ws::async_app::{AsyncStream, AsyncWebsocketApp};
use humphrey_ws::message::Message;

use std::sync::Arc;

fn main() {
    let websocket_app: AsyncWebsocketApp<()> = AsyncWebsocketApp::new()
        .with_connect_handler(connect_handler)
        .with_message_handler(message_handler);

    websocket_app.run();
}

fn connect_handler(stream: AsyncStream, _: Arc<()>) {
    // TODO
}

fn message_handler(stream: AsyncStream, message: Message, _: Arc<()>) {
    stream.send(message);
}
```

## Broadcasting Messages from Event Handlers
Our connection handler needs to broadcast a message to all connected clients when a new client connects. This message will also be sent to the new client. The `AsyncStream` provides functionality for this, but as we'll see later, this is not the only way to broadcast messages.

Let's add this to our connection handler.

```rs
// --snip--

fn connect_handler(stream: AsyncStream, _: Arc<()>) {
    let message = Message::new(format!("Welcome, {}!", stream.peer_addr()));
    stream.broadcast(message);
}

// --snip--
```

It's as simple as that! If we test this with `websocat` and connect from a few terminals, you'll see that each message is correctly echoed back to the client, and new connections are announced to everyone.

## Sending Messages without an Event
Broadcasts can also be triggered without an event. This is useful for sending messages to all connected clients from a separate thread, or for responding to non-WebSocket events. In this example, we'll broadcast the standard input to all connected clients.

To do this, we'll use an `AsyncSender`, which allows us to send messages and broadcasts without waiting for an event. Let's get a new async sender from the app, and send it to a separate thread for handling user input.

```rs
// --snip--

use humphrey_ws::async_app::AsyncSender;

use std::thread::spawn;

fn main() {
    let websocket_app: AsyncWebsocketApp<()> = AsyncWebsocketApp::new()
        .with_connect_handler(connect_handler)
        .with_message_handler(message_handler);

    let sender = websocket_app.sender();
    spawn(move || user_input(sender));

    websocket_app.run();
}

fn user_input(sender: AsyncSender) {
  // TODO
}

// --snip--
```

You can create as many senders as you want from the app, but they can only be created from the main thread and must be created before the application is run.

## Using the Sender
Now that we have a sender, we can use it to send messages to all connected clients. Let's use the same code from our synchronous example, but slightly modify it to work with a sender instead of the bus.

```rs
// --snip

use std::io::BufRead;

// --snip--

fn user_input(sender: AsyncSender) {
    let stdin = std::io::stdin();
    let handle = stdin.lock();

    for line in handle.lines().flatten() {
        sender.broadcast(Message::new(line));
    }
}

// --snip--
```

If we run this code now, every line we type in the server console will be broadcast to all connected clients.

## Full Example
The full source code for this example should look like this.

```rs
use humphrey_ws::async_app::{AsyncStream, AsyncWebsocketApp, AsyncSender};
use humphrey_ws::message::Message;

use std::io::BufRead;
use std::sync::Arc;
use std::thread::spawn;

fn main() {
    let websocket_app: AsyncWebsocketApp<()> = AsyncWebsocketApp::new()
        .with_connect_handler(connect_handler)
        .with_message_handler(message_handler);

    let sender = websocket_app.sender();
    spawn(move || user_input(sender));

    websocket_app.run();
}

fn user_input(sender: AsyncSender) {
    let stdin = std::io::stdin();
    let handle = stdin.lock();

    for line in handle.lines().flatten() {
        sender.broadcast(Message::new(line));
    }
}

fn connect_handler(stream: AsyncStream, _: Arc<()>) {
    let message = Message::new(format!("Welcome, {}!", stream.peer_addr()));
    stream.broadcast(message);
}

fn message_handler(stream: AsyncStream, message: Message, _: Arc<()>) {
    stream.send(message);
}
```

## Conclusion
In this chapter, we've learnt how to broadcast messages asynchronously. It's a lot easier than the synchronous approach, and also more flexible. In the next chapter, we'll learn how to integrate an asynchronous WebSocket application with an existing Humphrey application.
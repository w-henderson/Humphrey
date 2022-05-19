# Getting Started
This chapter will walk you through building a basic asynchronous WebSocket server with Humphrey WebSocket.

## Creating a New Project
For this example, we'll be building a WebSocket-only application, so we won't link it to a Humphrey Core application. If you want to use asynchronous WebSocket alongside an existing Core application, or if you want to use Humphrey's TLS integration, read the [Using with an Existing Humphrey App](linking.md) section.

Let's create a new project with `cargo new async_ws` and then add Humphrey WebSocket as a dependency in the `Cargo.toml` file. Make sure you replace the "*" version with the latest version of the crate.

```toml
[dependencies]
humphrey_ws = "*"
```

## Setting up the Application
Creating a new Humphrey WebSocket app looks very similar to creating a new Humphrey app. The `AsyncWebsocketApp` struct, like Humphrey's `App`, has one type parameter for the app's state, and is configured with a builder method. Unless [otherwise specified](linking.md), the app will manage its own Humphrey Core application behind-the-scenes, and will automatically respond to WebSocket requests to any route.

Let's set up the app with all the handlers we need.

```rs
use humphrey_ws::async_app::{AsyncStream, AsyncWebsocketApp};
use humphrey_ws::message::Message;

use std::sync::Arc;

fn main() {
    let websocket_app: AsyncWebsocketApp<()> = AsyncWebsocketApp::new()
        .with_connect_handler(connect_handler)
        .with_disconnect_handler(disconnect_handler)
        .with_message_handler(message_handler);

    websocket_app.run();
}

fn connect_handler(stream: AsyncStream, _: Arc<()>) {
    // TODO
}

fn disconnect_handler(stream: AsyncStream, _: Arc<()>) {
    // TODO
}

fn message_handler(stream: AsyncStream, message: Message, _: Arc<()>) {
    // TODO
}
```

This code will compile, run, and accept WebSocket connections, but it won't do anything with them yet. The connect and disconnect handlers are event handlers, and they are passed an `AsyncStream` and the app's state, which we ignore. The message handler is, you guessed it, a message handler, and it is passed an `AsyncStream`, the `Message` which triggered it, and the app's state, which again we ignore.

But what is an `AsyncStream`?

## What is an `AsyncStream`?
The `AsyncStream` struct is how Humphrey WebSocket represents an internal client connection, without giving the handler access to the underlying stream. It provides all the functionality required to send and broadcast messages, as well as the address of the client. It communicates with the actual client through a channel, which is read from by the main thread, which forwards messages to their corresponding clients. You don't need to think about any of this, however, since the asynchronous app's runtime will handle all of the details for you.

If, for whatever reason, you need access to the raw underlying stream, you'll need to use the synchronous WebSocket architecture described in the previous subsection.

## Implementing the Handlers
Let's now implement the handlers we defined earlier. For connections, we're going to send a welcome message, and for messages, we're going to respond with an acknowledgement. We're also going to print each event to the console.

```rs
// --snip--

fn connect_handler(stream: AsyncStream, _: Arc<()>) {
    println!("{}: Client connected", stream.peer_addr());

    stream.send(Message::new("Hello new client!"));
}

fn disconnect_handler(stream: AsyncStream, _: Arc<()>) {
    println!("{}: Client disconnected", stream.peer_addr());
}

fn message_handler(stream: AsyncStream, message: Message, _: Arc<()>) {
    println!(
        "{}: Message received: {}",
        stream.peer_addr(),
        message.text().unwrap().trim()
    );

    stream.send(Message::new("Message received!"));
}
```

You'll see we use the `Message` type to represent messages, which we discussed when we were talking about the synchronous WebSocket architecture. This is simply an abstraction over WebSocket frames and messages.

If we run this code now and connect to it with `websocat` (which we learnt about [here](../sync/getting-started.md#testing-our-websocket-handler-optional)), it should run as expected.

**Client:**
```bash
william@pc:~$ websocat ws://127.0.0.1
Hello new client!
Example message
Message received!
^C
```

**Server:**
```
127.0.0.1:50189: Client connected
127.0.0.1:50189: Message received: Example message
127.0.0.1:50189: Client disconnected
```

## Detecting Unexpected Disconnections
Generally, when a client disconnects, they gracefully close the connection by sending a "close" frame to the server. However, if the client disconnects suddenly, such as in the case of a loss of network connectivity, not only will the WebSocket connection not be closed, but the underlying TCP stream won't be either. This means that the disconnect handler will not be called, which could cause issues in your application.

Fortunately, Humphrey WebSocket provides a way around this by way of heartbeats. A heartbeat consists of a ping and a pong, the former being sent from the server to the client, and vice versa. An asynchronous WebSocket application can be configured to send a ping every `interval` seconds, and the client will automatically respond with a pong. If no pongs are received in `timeout` seconds, the connection will be closed and the disconnect handler correctly called.

To do this in our example, we'll simply need to make a small change when we first create our app. We'll be using 5 seconds for our ping interval and 10 seconds for our timeout.

```rs
use humphrey_ws::async_app::{AsyncStream, AsyncWebsocketApp};
use humphrey_ws::message::Message;
use humphrey_ws::ping::Heartbeat;

use std::sync::Arc;
use std::time::Duration;

fn main() {
    let websocket_app: AsyncWebsocketApp<()> = AsyncWebsocketApp::new()
        .with_heartbeat(Heartbeat::new(Duration::from_secs(5), Duration::from_secs(10)))
        .with_connect_handler(connect_handler)
        .with_disconnect_handler(disconnect_handler)
        .with_message_handler(message_handler);

    websocket_app.run();
}

// --snip--
```

## Conclusion
In this chapter, we've learnt about sending and receiving WebSocket messages asynchronously. Next, we'll learn how to broadcast messages to all connected clients, and compare this to how we did it synchronously.
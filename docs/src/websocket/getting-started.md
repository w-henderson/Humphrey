# Getting Started
This chapter will walk you through the steps to get started with Humphrey WebSocket.

## Adding WebSocket Support to a Humphrey Project
To add WebSocket support to an existing project, you just need to add the `humphrey_ws` dependency to your `Cargo.toml` file. It is important to ensure that the version is acceptable for that of the core crate, so you should ideally find out the latest version for each and fix the version accordingly.

```toml
[dependencies]
humphrey = "*"
humphrey_ws = "*"
```

If you want to create a new project with WebSocket support, first follow the instructions in the [Humphrey Core section](../core/getting-started.md) to create a new Humphrey project, then add the `humphrey_ws` dependency.

## Setting up a WebSocket Handler
To add a WebSocket route to your Humphrey app, use the `with_websocket_route` method on the `App` struct, providing the path to match and the handler, just like you would with any other route. However, you should wrap the handler function in this crate's `websocket_handler` function, which will allow it to handle the WebSocket handshake behind-the-scenes.

Let's create a new `App` struct and add a route to match any path.

```rs
use humphrey::stream::Stream;
use humphrey::App;

use humphrey_ws::stream::WebsocketStream;
use humphrey_ws::websocket_handler;

use std::sync::Arc;

fn main() {
    let app: App = App::new()
        .with_websocket_route("/*", websocket_handler(my_handler));
        
    app.run("0.0.0.0:80").unwrap();
}

fn my_handler(mut stream: WebsocketStream<Stream>, _: Arc<()>) {
    println!("Connection from {:?}", stream.inner().peer_addr().unwrap());

    // TODO: Implement handler
}
```

If you run this code, the app will start, but all WebSocket connections will be immediately closed after printing their addresses since the handler function immediately returns and thus the stream is dropped. This can be a useful feature of the `WebsocketStream` type, since the client is automatically sent a "close" frame when it is dropped.

## Testing our WebSocket Handler (optional)
In production, it is likely that our application would only ever be accessed from a browser. However, during development, it can be useful to connect to the server from a terminal with a tool like netcat for debugging. We'll use [`websocat`] for this, which is a simple Rust CLI to do exactly this. It can be installed with `cargo install websocat`.

Let's connect to our server.

```sh
$ websocat ws://127.0.0.1/
```

The connection will not immediately close, but it will be closed if you attempt to send a message. The running server will however print a message to the console to indicate that the connection was successful.

## Receiving Messages
Messages can be received from the client in three ways. Firstly, you can use the `recv` method on the stream to block until a message is received or an error is encountered. Secondly, you can use `recv_nonblocking` to check if a message is available without blocking, which will be discussed [next](broadcasting-messages.md). Finally, you can make use of the stream's implementation of the `Read` trait, which allows you to use the stream with Rust's built-in functions. For this example, we'll use the first method.

Let's change our code so it continually listens for messages, and prints them to the console.

```rs
// --snip--

fn my_handler(mut stream: WebsocketStream<Stream>, _: Arc<()>) {
    let address = stream.inner().peer_addr().unwrap();

    println!("{:?}: <connected>", address);

    while let Ok(message) = stream.recv() {
        println!("{:?}: {}", address, message.text().unwrap().trim());
    }

    println!("{:?}: <disconnected>", address);
}
```

Now, we loop while we are successfully receiving messages, and print each one to the console. The `message.text()` function converts each message to a string, which will return an error if the message is not valid UTF-8. However, we don't need to worry about this since we are only sending text messages.

If we connect to the server again using `websocat`, we can test our code.

```sh
$ websocat ws://127.0.0.1/
hello world
this is working
```

We should see the following output in the console:

```
127.0.0.1:12345: <connected>
127.0.0.1:12345: hello world
127.0.0.1:12345: this is working
127.0.0.1:12345: <disconnected>
```

## Sending Messages
Messages can be sent to the client in either of two ways. We can either use the `send` method on the stream to send a message, or we can use the stream's implementation of the `Write` trait. Since we used the corresponding `recv` method earlier, we'll use the former.

Now we're going to modify our code so that it echoes back each message to the client after printing it to the console, as well as sending an initial "Hello, world! message when each client first connects.

```rs
// --snip--

use humphrey_ws::message::Message;

// --snip--

fn my_handler(mut stream: WebsocketStream<Stream>, _: Arc<()>) {
    let address = stream.inner().peer_addr().unwrap();

    println!("{:?}: <connected>", address);

    stream.send(Message::new("Hello, world!")).unwrap();

    while let Ok(message) = stream.recv() {
        println!("{:?}: {}", address, message.text().unwrap().trim());

        stream.send(message).unwrap();
    }

    println!("{:?}: <disconnected>", address);
}
```

When the client first connects, we use the `Message::new` constructor to create a new message and then send it to the client with `stream.send`. The message will automatically be marked as a text message since the payload is valid UTF-8. If we were to send it as a binary message, we would use `Message::new_binary`, or supply a non-UTF-8 payload to the regular constructor.

You can now use `websocat` again to test your code.

## Conclusion
In this chapter, we've learnt about sending and receiving WebSocket messages within a Humphrey application. Next, let's look at the [Broadcasting Messages](broadcasting-messages.md) chapter, which cover how to use non-blocking reads to create a simple broadcast server.
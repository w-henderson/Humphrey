# Using with an Existing Humphrey App
An asynchronous WebSocket application can be linked to a Humphrey application two ways, internally or externally. So far, we've only dealt with internal linking, which is where the WebSocket application manages its own Humphrey application. However, in many cases it might be more convenient to use a WebSocket application as part of a larger Humphrey Core application, and this is required if you want to use TLS.

For this chapter only, we'll start by looking at the entire code for this example, then learn how it works.

## The Code
```rs
use humphrey::http::{Response, StatusCode};
use humphrey::App;

use humphrey_ws::async_app::{AsyncStream, AsyncWebsocketApp};
use humphrey_ws::handler::async_websocket_handler;
use humphrey_ws::message::Message;

use std::sync::Arc;
use std::thread::spawn;

fn main() {
    let websocket_app: AsyncWebsocketApp<()> =
        AsyncWebsocketApp::new_unlinked().with_message_handler(message_handler);

    let humphrey_app: App<()> = App::new()
        .with_stateless_route("/", |_| Response::new(StatusCode::OK, "Hello world!"))
        .with_websocket_route(
            "/ws",
            async_websocket_handler(websocket_app.connect_hook().unwrap()),
        );

    spawn(move || humphrey_app.run("0.0.0.0:80").unwrap());

    websocket_app.run();
}

fn message_handler(stream: AsyncStream, message: Message, _: Arc<()>) {
    stream.send(message);
}
```

## Creating a New, Unlinked `AsyncWebsocketApp`
A WebSocket application is considered to be unlinked if it doesn't have a link to an existing Humphrey application. We can create a new, unlinked WebSocket application by calling `AsyncWebsocketApp::new_unlinked()`. This requires the user to link the application to a Humphrey application rather than allowing the WebSocket application to manage it internally.

Running an unlinked WebSocket application will not throw an error, but it will not be able to receive messages.

## What is a Connect Hook?
We link a WebSocket application to a Humphrey application using a connect hook, which is effectively the sending end of a channel which sends new WebSocket connections from the Humphrey application to the WebSocket application. Humphrey Core processes the incoming "upgrade" HTTP request, Humphrey WebSocket completes the handshake, and then your WebSocket application takes it from there.

## Asynchronous WebSocket Routes
To define a WebSocket route on the Humphrey application as an entry point to your WebSocket application, we use the `async_websocket_handler` function, which provides a convenient way of performing the handshake and then passing the connection to your asynchronous application.

This function takes a connect hook as its argument, and returns the handler function for the route.

## Running Both Apps
Since the WebSocket application does not manage its own Humphrey application, we need to run both apps in separate threads. It doesn't matter which runs first or which runs on the main thread, but as soon as the Humphrey application is started, new WebSocket connections are able to accumulate in through the connect hook, which could cause a performance issue if the WebSocket application is not started straight away.

In our code, we run the Humphrey application first on a new thread, and then the WebSocket application on the main thread.

## Conclusion
In this chapter, we've seen how to create an unlinked WebSocket application and manually link it to a Humphrey application. If you want to learn more about Humphrey WebSocket, consider taking a look at the [API reference](https://docs.rs/humphrey-ws).
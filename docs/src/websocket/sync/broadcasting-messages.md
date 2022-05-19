# Broadcasting Messages
It's common in a WebSocket application to broadcast messages to many clients at once, so in this chapter we'll learn how to do this using Humphrey WebSocket. We will have to use an external dependency `bus` to provide a single-producer, multiple-consumer channel to send messages to the client handler threads to then be sent on to each client.

The example we build in this chapter will simply echo messages back to the client like we did before, but with the addition that any messages typed into the server console will be broadcast to all connected clients.

## Initialising the Project
As before, we need a new Humphrey application, along with the following dependencies:

```toml
[dependencies]
humphrey = "*"
humphrey_ws = "*"
bus = { git = "https://github.com/agausmann/bus", branch = "read_handle/lock" }
```

You'll notice that the `bus` dependency is specified with a GitHub address. This is because we need to be able to add readers to the bus from different threads, and this functionality is not yet merged into the main crate, so we need to use [Adam Gausmann](https://github.com/agausmann)'s fork.

Let's copy the code we used at the start of the last chapter to create a new WebSocket-enabled application:

```rs
use humphrey::App;

use humphrey_ws::stream::WebsocketStream;
use humphrey_ws::websocket_handler;

use std::sync::Arc;

fn main() {
    let app: App = App::new()
        .with_websocket_route("/*", websocket_handler(my_handler));
        
    app.run("0.0.0.0:80").unwrap();
}

fn my_handler(mut stream: WebsocketStream, _: Arc<()>) {
    // TODO: Implement handler
}
```

## Initialising the Bus
This time, we need to share some state between the handlers: the bus. We'll define the state type as simply a mutex around a read handle to the bus. This will only need to be locked very briefly when each client first connects in order to add a reader to the bus. We also need to create the bus and a read handle to it. Let's make these changes:

```rs
// --snip--

use std::sync::{Arc, Mutex};

use bus::{Bus, BusReadHandle};

type AppState = Mutex<BusReadHandle<String>>;

fn main() {
    let bus: Bus<String> = Bus::new(16);
    let read_handle = bus.read_handle();

    let app: App<AppState> = App::new_with_config(32, Mutex::new(read_handle))
        .with_websocket_route("/*", websocket_handler(my_handler));

    app.run("0.0.0.0:80").unwrap();
}

fn my_handler(mut stream: WebsocketStream, read_handle: Arc<AppState>) {
    // TODO: Implement handler
}
```

You'll see that we also changed `App::new` to `App::new_with_config` to specify the initial state value. This is because we need to pass the read handle to the app, so it can share it with the handlers. We also have to specify the number of threads to use as part of this more flexible constructor.

## Non-Blocking Reads
Next, we need to effectively read messages from the stream and the bus at the same time. We can't do this, so we use non-blocking reads to attempt to read from the stream without blocking, then do the same with the bus.

The `recv_nonblocking` function of the stream returns a `Restion`, which is an enum merging the core `Result` and `Option` types, giving it variants `Ok(value)`, `Err(error)` and `None`. The `None` variant indicates that the read was successful, but there was nothing to read.

Let's implement this in the code:

```rs
// --snip--

use std::thread::sleep;
use std::time::Duration;

// --snip--

fn my_handler(mut stream: WebsocketStream, read_handle: Arc<AppState>) {
    let mut rx = { read_handle.lock().unwrap().add_rx() };

    loop {
        match stream.recv_nonblocking() {
            Restion::Ok(message) => stream.send(message).unwrap(),
            Restion::Err(_) => break,
            Restion::None => (),
        }

        if let Ok(channel_message) = rx.try_recv() {
            stream.send(Message::new(channel_message)).unwrap();
        }

        sleep(Duration::from_millis(64));
    }
}
```

We first temporarily lock the mutex to create a new bus reader, then continuously attempt to read from the stream and the bus. If the read from the stream was successful, we echo back the message. If an error occurred, we close the connection, and if no message was read, we do nothing. Then, we do the same with the bus, and if the read was successful, we send the broadcasted message to the client. Finally, we sleep for a short time to avoid busy-waiting.

If you run the server now and test it with `websocat`, it will behave exactly like the server we built in the previous chapter.

## Broadcasting User Input
Now our handlers are set up, we just need to give them something to broadcast. For this, we can simply read the standard input and send it line by line to the bus. This will have to take place on a separate thread, since the Humphrey application blocks the main thread indefinitely.

This can be simply implemented as follows:

```rs
// --snip--

use std::io::BufRead;
use std::thread::{sleep, spawn};

// --snip--

fn main() {
    let bus: Bus<String> = Bus::new(16);
    let read_handle = bus.read_handle();

    spawn(move || main_thread(bus));

    // --snip --
}

fn main_thread(mut bus: Bus<String>) {
    let stdin = std::io::stdin();
    let handle = stdin.lock();

    for line in handle.lines().flatten() {
        bus.broadcast(line);
    }
}
```

## Testing the Server
Let's open up three terminal windows, and run the server on one of them. In the other two, connect to the server with `websocat` as we did before with `websocat ws://127.0.0.1/`. If you send messages to the server in either of the client terminal, you'll see that they are individually echoed back to the client. However, if you type a message in the server terminal, you'll see it broadcasted to both connected clients. It works!

## Full Example
The full source code for this example should look like this:

```rs
use humphrey::App;

use humphrey_ws::message::Message;
use humphrey_ws::restion::Restion;
use humphrey_ws::stream::WebsocketStream;
use humphrey_ws::websocket_handler;

use bus::{Bus, BusReadHandle};

use std::io::BufRead;
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};
use std::time::Duration;

type AppState = Mutex<BusReadHandle<String>>;

fn main() {
    let bus: Bus<String> = Bus::new(16);
    let read_handle = bus.read_handle();

    spawn(move || main_thread(bus));

    let app: App<AppState> = App::new_with_config(32, Mutex::new(read_handle))
        .with_websocket_route("/*", websocket_handler(my_handler));

    app.run("0.0.0.0:80").unwrap();
}

fn main_thread(mut bus: Bus<String>) {
    let stdin = std::io::stdin();
    let handle = stdin.lock();

    for line in handle.lines().flatten() {
        bus.broadcast(line);
    }
}

fn my_handler(mut stream: WebsocketStream, read_handle: Arc<AppState>) {
    let mut rx = { read_handle.lock().unwrap().add_rx() };

    loop {
        match stream.recv_nonblocking() {
            Restion::Ok(message) => stream.send(message).unwrap(),
            Restion::Err(_) => break,
            Restion::None => (),
        }

        if let Ok(channel_message) = rx.try_recv() {
            stream.send(Message::new(channel_message)).unwrap();
        }

        sleep(Duration::from_millis(64));
    }
}
```

## Conclusion
Humphrey WebSocket provides powerful WebSocket support for Humphrey applications. When paired with other crates, like the `bus` crate here, it can be used for even more complex tasks with minimal code. We'll now take a look at how to do this asynchronously.
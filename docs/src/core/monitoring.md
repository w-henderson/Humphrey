# Monitoring Events
In this chapter, we'll discuss how to monitor internal events in the application. It can often be useful to log events such as requests and errors, which can be useful for debugging and for general performance analysis. To learn about this, we're going to build a simple logger which logs all events in a "Hello, world!" application to both the console, and specific events to a file.

## Setting up our Application
We'll start with an extremely simple application which simply responds with "Hello, world!" to every request. After creating a new crate and adding Humphrey as a dependency, as outlined in [Getting Started](getting-started.md), add the following code to the main file.

```rs
use humphrey::http::{Response, StatusCode};
use humphrey::App;

fn main() {
    let app: App =
        App::new().with_stateless_route("/*", |_| Response::new(StatusCode::OK, "Hello, world!"));

    app.run("0.0.0.0:80").unwrap();
}
```

This will just return "Hello, world!" to every request.

## Logging to the Console
Monitoring events in Humphrey is done over a channel of `Event`s. An `Event` is a simple struct which contains the event's type, as well as an optional address of the client and an optional string with additional information. We'll cover this in more detail later, but for now we just need to know that it implements the `Display` trait so that we can print it.

Monitoring is configured using the `MonitorConfig` struct, and events can be subscribed to using its `with_subscription_to` function. You can subscribe to a single event or an event level (such as warning), since both types implement the `ToEventMask` trait.

Let's create a channel and supply it to the application using the `with_monitor` method, as well as subscribing to all events using the debug event level. We'll also create a thread to listen on the channel and print all events to the console.

```rs
use humphrey::http::{Response, StatusCode};
use humphrey::monitor::event::EventLevel;
use humphrey::monitor::MonitorConfig;
use humphrey::App;

use std::sync::mpsc::channel;
use std::thread::spawn;

fn main() {
    let (tx, rx) = channel();

    let app: App = App::new()
        .with_monitor(MonitorConfig::new(tx).with_subscription_to(EventLevel::Debug))
        .with_stateless_route("/*", |_| Response::new(StatusCode::OK, "Hello, world!"));

    spawn(move || {
        for e in rx {
            println!("{}", e);
        }
    });

    app.run("0.0.0.0:80").unwrap();
}
```

If we run this application and visit it in the browser, you'll see a lot of debug output in the console. You have successfully logged some internal events!

## Filtering Events
Events should always be filtered at the `MonitorConfig` level if possible to reduce the traffic on the channel. However, if, for example, you want to print everything to the console but only write warnings and errors to a file, you can filter events using event masks.

An event mask is simply a `u32` which is a bit mask of the events you want to subscribe to. As an example, `EventLevel::Debug` is simply `0xFFFFFFFF`, which means that all events are subscribed to. The individual event `EventType::RequestServedSuccess` is `0x00000040`. You probably don't need to know the bit mask values, but they are useful for understanding how event filtering works.

Let's move our event listening thread to a new function, and temporarily filter out all events except warnings and errors.

```rs
use std::sync::mpsc::{channel, Receiver};
use humphrey::monitor::event::{Event, EventLevel};

// --snip--

spawn(move || monitor_thread(rx));

// --snip--

fn monitor_thread(rx: Receiver<Event>) {
    for e in rx {
        println!("{}", e);
    }
}
```

Now that our thread is in a new function, we can add the following code to filter out all events except warnings and errors.

```rs
fn monitor_thread(rx: Receiver<Event>) {
    for e in rx {
        if e.kind as u32 & EventLevel::Warning as u32 != 0 {
            println!("{}", e);
        }
    }
}
```

If you run the program now, you'll probably see no output in the console, as none of the events being received are warnings or errors. If you use a tool like Netcat to send an invalid request to the application, you'll see an error message.

## Writing Events to a File
Let's add a little bit more code to the monitor thread to write all events to a file.

```rs
use std::fs::File;
use std::io::Write;

// --snip--

fn monitor_thread(rx: Receiver<Event>) {
    let mut file = File::create("monitor.log").unwrap();

    for e in rx {
        if e.kind as u32 & EventLevel::Warning as u32 != 0 {
            file.write_all(format!("{}\n", e).as_bytes()).unwrap();
        }

        println!("{}", e);
    }
}
```

If we run the code again, we'll see that all of our events are again logged to the console, but warnings and errors are additionally logged to the file. Since our program isn't producing any errors at the moment, let's intentionally cause some by getting a thread to panic!

## Monitoring Thread Panics
**Threads should not ever panic.** However, in any application there's always a chance of bugs causing threads to panic, and this should not cause the whole program to stop working. Humphrey automatically detects thread panics in the background and quietly restarts the affected thread, while letting Rust log the panic to the console in the typical way.

If your monitor is subscribed to the `EventType::ThreadPanic` event, whether directly or through any event level, Humphrey will take over the panic logging from Rust, and will send the panic to the monitor channel instead of printing it to the standard error stream. It's important that this is logged in some way, as you should never miss a panic!

Let's cause a panic on the route "/panic" by adding another simple handler.

```rs
// --snip--

let app: App = App::new()
    .with_monitor(MonitorConfig::new(tx).with_subscription_to(EventLevel::Debug))
    .with_stateless_route("/panic", |_| panic!("this is a panic"))
    .with_stateless_route("/*", |_| Response::new(StatusCode::OK, "Hello, world!"));

// --snip--
```

If you visit the panic route in your browser now, you won't get a response from the server as the thread has panicked, but you'll see the panic in the console and the file, as well as that the thread was restarted in the console.

## Conclusion
In conclusion, Humphrey provides a flexible way for logging internal events. Next, we'll look at how to use Humphrey with the Tokio async runtime.
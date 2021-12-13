<script>
    window.setInterval(() => {
        document.querySelector("#datetime-example").innerHTML = new Date().toUTCString();
    }, 1000);
</script>

# Getting Started
This chapter will walk you through the steps to get started with Humphrey Core. It assumes that you already have Rust and Cargo installed, but installation steps for these can be found in the [Rust book](https://doc.rust-lang.org/book/ch01-01-installation.html).

## Creating a New Project
A Humphrey Core web application is a Rust binary crate, so to begin we'll need to create a new project using Cargo.

```bash
$ cargo new my-app
```

Next, we need to add the `humphrey` crate as a dependency of our project, which can be done by editing the `Cargo.toml` file within the project directory as follows. Please note that it is not good practice to use the `*` version number for a real project, as if a new version of Humphrey adds breaking changes, this could cause your application to stop working properly. In that case, you should check the latest Humphrey version on [crates.io](https://crates.io/crates/humphrey) and use that version number.

```toml
[dependencies]
humphrey = "*"
```

With that, our project will compile and run, but at the moment it doesn't do anything.

## Creating a Humphrey App
We can now initialise a new Humphrey application in the `main.rs` file of the `src` directory.

```rs
use humphrey::http::{Response, StatusCode};
use humphrey::App;

fn main() {
    let app: App = App::new().with_stateless_route("/", |request| {
        Response::new(StatusCode::OK, "Hello, Humphrey!", &request)
    });

    app.run("0.0.0.0:80").unwrap();
}
```

If we now run `cargo run`, our application will successfully compile and start the server, which you can access at [http://localhost](http://localhost). You should see the text "Hello, Humphrey!" in your browser. If so, congratulations - you've successfully created a Humphrey web application! Let's go into a little more detail about what this code actually does.

First, we create a new `App` instance, which is the core of every Humphrey application. We need to specify the type `App` as well, since the app is generic over a state type, which we'll cover in the [Using State](state.md) chapter. This shouldn't be necessary since the default state type is the Rust empty type `()`, but it must be done due to current technical limitations of Rust (see [rust-lang/rust issue #36887](https://github.com/rust-lang/rust/issues/36887)).

We then call `with_stateless_route` on the `App` instance, passing in the path of the route and a closure that will be called when the route is matched. The closure takes one argument, the request. It returns a `Response` object with the success status code (200), the text "Hello, Humphrey!", and takes in the request to ensure that the response is constructed correctly, for example respecting the `Connection` header.

Finally, we call `run` on the `App` instance, passing in the address and port to listen on. This will start the server and block the thread until the server is shut down.

## Adding Multiple Routes
At the moment, our app only shows a message for the root path, but we can add more routes by calling `with_stateless_route` again with different handlers. In most cases, these would not be passed in as closures, but rather as functions that return a `Response` object. Let's add another route called `/api/time` that shows the current time.

We'll start by creating a handler function and adding it to the app by calling `with_stateless_route` again. We'll also move the root handler to a function as well to improve the code readability. We also need to import the `Arc` type.

```rs
use humphrey::http::{Request, Response, StatusCode};
use humphrey::App;
use std::sync::Arc;

fn main() {
    let app: App = App::new()
        .with_stateless_route("/", root_handler)
        .with_stateless_route("/api/time", time_handler);

    app.run("0.0.0.0:80").unwrap();
}

fn root_handler(request: Request) -> Response {
    Response::new(StatusCode::OK, "Hello, Humphrey!", &request)
}

fn time_handler(request: Request) -> Response {
    // todo: get the current time
}
```

This code won't compile, as the `time_handler` function does not yet return a `Response` object. Let's use the built-in `DateTime` type from `humphrey::http::date` to get the current time in the HTTP date format, which looks like <code class="hljs" id="datetime-example"><script>document.write(new Date().toUTCString());</script></code>.

```rs
use humphrey::http::date::DateTime;

// --snip--

fn time_handler(request: Request) -> Response {
    let time = DateTime::now();
    let time_http = time.to_string();

    Response::new(StatusCode::OK, time_http, &request)
}
```

If we now run `cargo run` again, and go to [http://localhost/api/time](http://localhost/api/time) in the browser, we should see the current time in the format described earlier.

## Wildcard Routes
Humphrey supports the `*` wildcard character in route paths, so one handler can handle many paths. Let's add another route called `/api/greeting/*` which will greet the user with the name they provide. Again, we'll need to create another route handler function and add it to the app:

```rs
// --snip--

fn main() {
    let app: App = App::new()
        .with_stateless_route("/", root_handler)
        .with_stateless_route("/api/time", time_handler);
        .with_stateless_route("/api/greeting/*", greeting_handler);

    app.run("0.0.0.0:80").unwrap();
}

// --snip--

fn greeting_handler(request: Request) -> Response {
    // todo: greet the user
}
```

In our newly-created greeting handler, we want to extract the name from the path and return a response depending on the name provided. We can do that with the Rust standard library's `strip_prefix` function for strings.

```rs
// --snip--

fn greeting_handler(request: Request) -> Response {
    let name = request.uri.strip_prefix("/api/greeting/").unwrap();
    let greeting = format!("Hello, {}!", name);

    Response::new(StatusCode::OK, greeting, &request)
}
```

If we now visit [http://localhost/api/greeting/Humphrey](http://localhost/api/greeting/Humphrey) in the browser, we should see the text "Hello, Humphrey!". You can replace the name Humphrey with your own name or any other name you want, and you should see the greeting change accordingly.

## Conclusion
As you can see, Humphrey provides an intuitive and easy-to-use API to create web applications. Next, let's look at the [Using State](state.md) chapter, which will cover how to safely share state between routes and requests.
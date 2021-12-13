# Using State
This chapter covers the basics of sharing state between requests to a Humphrey web application. In this chapter, we will demonstrate how to use the `App`'s `State` type parameter to share state between requests by building a simple application which displays a button and how many times it has been clicked.

Basic knowledge of JavaScript is useful to fully understand this chapter.

## Creating a Stateful App
Once you've created an empty Rust project with the Humphrey dependency installed, as described in the previous chapter, you'll need to define a struct to hold the state of your application.

```rs
use humphrey::handlers::serve_dir;
use humphrey::http::{Request, Response, StatusCode};
use humphrey::App;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Default)]
struct AppState {
    button_presses: AtomicUsize,
}

fn main() {}
```

You'll notice that we derive the trait `Default` on our state struct. This is not required, but it means we don't need to explicitly define the initial state of the application in our `main` function, as it will be set to zero button presses.

We can now create our `App` instance in the main function with three routes, one API endpoint to get the current number of button presses, one which increments this number by one, and a catch-all route at the bottom which serves the `static` directory if none of the other endpoints are matched. You'll see that we use the `serve_dir` built-in handler with the `with_path_aware_route` method, which you can read about further in the next section. We also use the `with_route` method instead of `with_stateless_route`, since we want access to the app's state.

```rs
// --snip--

fn main() {
    let app: App<AppState> = App::new()
        .with_route("/api/getPresses", get_presses)
        .with_route("/api/incrementPresses", increment_presses)
        .with_path_aware_route("/*", serve_dir("./static"));

    app.run("0.0.0.0:80").unwrap();
}
```

## Defining the API Endpoints
We now need to create the two API endpoints which get and increment the button presses. If you are familiar with Rust, you'll know that the `AtomicUsize` type makes it very easy to share and increment a number between threads.

Our `/api/getPresses` endpoint just needs to load the value of the `button_presses` field from the state and return it as the response body, as follows. We use `Ordering::SeqCst` to ensure that the value is sequentially consistent, which means that every subsequent call of the API will never be less than the value returned by the previous call.

```rs
// --snip--

fn get_presses(request: Request, state: Arc<AppState>) -> Response {
    let presses = state.button_presses.load(Ordering::SeqCst);

    Response::new(StatusCode::OK, presses.to_string(), &request)
}
```

Creating the `/api/incrementPresses` endpoint is similar, but we need to increment the value of the `button_presses` field instead of returning it. This is done as follows.

```rs
/// --snip--

fn increment_presses(request: Request, state: Arc<AppState>) -> Response {
    state.button_presses.fetch_add(1, Ordering::SeqCst);

    Response::new(StatusCode::OK, b"OK", &request)
}
```

## Creating a Simple Front-End
We now need to create a basic HTML page with a button and some text to interface with our Humphrey application. We can do this with some simple HTML and JavaScript as follows.

**`index.html`**
```html
<html>

<head>
  <title>Humphrey Stateful Tutorial</title>
</head>

<body>
  <h1>Button has been pressed <span id="presses">x</span> times</h1>
  
  <button onclick="incrementPresses()">Press Me</button>

  <script src="index.js"></script>
</body>

</html>
```

**`index.js`**
```js
function updatePresses() {
  fetch("/api/getPresses").then(res => res.text())
    .then(text => {
      document.querySelector("#presses").innerHTML = text;
    });
}

function incrementPresses() {
  fetch("/api/incrementPresses").then(updatePresses);
}

window.onload = updatePresses;
```

This code simply fetches the current number of button presses from the API and updates the page accordingly. It also shows a button which increments the number of button presses by one.

## Running our App
When we run `cargo run` in the terminal and visit [http://localhost](http://localhost) in the browser, we'll see the text "Button has been pressed 0 times" and a button which increments the number of button presses by one. If you press the button, you'll see the number increase. You can refresh the page or visit from a different device, and the number will be consistent.

## Conclusion
In this chapter, we've learnt how to create a stateful application with Humphrey. In the next chapter [Serving Static Content](static-content.md), we'll discuss the number of ways Humphrey provides to serve static content.
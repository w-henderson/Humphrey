use humphrey::http::{Request, Response, StatusCode};
use humphrey::App;
use std::error::Error;
use std::sync::{Arc, Mutex};

/// Struct representing the state of the app.
/// The `Default` trait must be derived or implemented for the `App` type.
#[derive(Default)]
struct AppState {
    /// Keeps track of the total button presses across threads
    button_presses: usize,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Initialise app using the `AppState` type as its state
    let app: App<AppState> = App::new()
        .with_route("/", index)
        .with_route("/press", press);

    // Run the app on localhost port 80
    app.run(&("127.0.0.1:80".parse()?))?;

    Ok(())
}

/// Request handler for the `/` path.
fn index(request: &Request, state: Arc<Mutex<AppState>>) -> Response {
    // Get the number of button presses using the thread-safe `Arc<Mutex<T>>`
    let state_locked = state.lock().unwrap();
    let presses = state_locked.button_presses;

    // Generate the HTML for the page and inject the number of button presses
    let html = include_str!("index.html").replace("{presses}", &presses.to_string());

    // Generate and return the response
    Response::new(StatusCode::OK) // code 200, success
        .with_bytes(html.as_bytes().to_vec()) // add the HTML as the response body
        .with_request_compatibility(request) // ensure that HTTP versions and `Connection` headers match the request
        .with_generated_headers() // automatically add required headers like `Date` and `Content-Length`
}

fn press(request: &Request, state: Arc<Mutex<AppState>>) -> Response {
    // Increment the number of button presses using the thread-safe `Arc<Mutex<T>>`
    let mut state_locked = state.lock().unwrap();
    state_locked.button_presses += 1;

    // Generate and return the response
    Response::new(StatusCode::OK)
        .with_bytes(b"OK".to_vec())
        .with_request_compatibility(request)
        .with_generated_headers()
}

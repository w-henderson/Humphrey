use humphrey::http::{Request, Response, StatusCode};
use humphrey::App;
use std::error::Error;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Struct representing the state of the app.
/// The `Default` trait must be derived so the app knows to start from 0 button presses.
#[derive(Default)]
struct AppState {
    /// Keeps track of the total button presses across threads
    button_presses: AtomicUsize,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Initialise app using the `AppState` type as its state
    let app: App<AppState> = App::new()
        .with_route("/", index)
        .with_route("/api/addPress", add_press)
        .with_route("/api/getPresses", get_presses);

    // Run the app on localhost port 80
    app.run("0.0.0.0:80")?;

    Ok(())
}

/// Request handler for the `/` path.
fn index(request: Request, state: Arc<AppState>) -> Response {
    // Get the number of button presses using the thread-safe `AtomicUsize`
    let presses = state.button_presses.load(Ordering::SeqCst);

    // Generate the HTML for the page and inject the number of button presses
    let html = include_str!("index.html").replace("{presses}", &presses.to_string());

    // Generate and return the response
    Response::new(StatusCode::OK, html, &request)
}

fn add_press(request: Request, state: Arc<AppState>) -> Response {
    // Increment the number of button presses using the thread-safe `AtomicUsize`
    state.button_presses.fetch_add(1, Ordering::SeqCst);

    // Generate and return the response
    Response::new(StatusCode::OK, b"OK", &request)
}

fn get_presses(request: Request, state: Arc<AppState>) -> Response {
    // Get the number of button presses using the thread-safe `AtomicUsize`
    let presses = state.button_presses.load(Ordering::SeqCst);

    // Generate and return the response
    Response::new(StatusCode::OK, presses.to_string(), &request)
}

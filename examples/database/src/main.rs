use humphrey::handlers::serve_dir;
use humphrey::http::method::Method;
use humphrey::http::{Request, Response, StatusCode};
use humphrey::App;

use jasondb::Database;

use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::UNIX_EPOCH;

type AppState = Mutex<Database<String>>;

fn main() -> Result<(), Box<dyn Error>> {
    // Open the database or create it if it doesn't exist.
    let database: Database<String> = Database::new("database.jdb")?;

    // Create an app with the database as the state.
    let app: App<AppState> = App::new_with_config(32, Mutex::new(database))
        // Add a handler for the root path since we'll need that to be dynamic.
        .with_route("/", home)
        // Add an API endpoint to add a message to the database.
        .with_route("/api/postMessage", post_message)
        // Serve every other route from the static directory.
        .with_path_aware_route("/*", serve_dir("./static"));

    // Start the server.
    app.run("0.0.0.0:80")?;

    Ok(())
}

/// The handler for the root path.
fn home(_: Request, db: Arc<AppState>) -> Response {
    // Query the database for all messages.
    let mut messages = db.lock().unwrap().iter().flatten().collect::<Vec<_>>();

    messages.sort_unstable_by(|(a, _), (b, _)| {
        a.parse::<u128>().unwrap().cmp(&b.parse::<u128>().unwrap())
    });

    let messages = messages
        .into_iter()
        .map(|(_, v)| v)
        .fold(String::new(), |acc, v| acc + &format!("<li>{}</li>", v));

    // Render the template with the messages.
    let html = include_str!("../static/index.html").replace("{messages}", &messages);

    Response::new(StatusCode::OK, html)
}

/// The handler for the API endpoint to add a message to the database.
fn post_message(request: Request, db: Arc<AppState>) -> Response {
    // If the request is not a POST request, return a 405 Method Not Allowed response.
    if request.method != Method::Post {
        return Response::new(StatusCode::MethodNotAllowed, "405 Method Not Allowed");
    }

    // Get the message from the request body.
    if let Some(body) = &request.content {
        let message = String::from_utf8(body.clone()).unwrap();

        // Add the message to the database with the current time as the key.
        let mut db = db.lock().unwrap();
        let key = UNIX_EPOCH.elapsed().unwrap().as_millis();
        db.set(key.to_string(), message).unwrap();
    }

    // Return a 200 OK response.
    Response::new(StatusCode::OK, b"OK")
}

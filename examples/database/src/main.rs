use humphrey::handlers::serve_dir;
use humphrey::http::method::Method;
use humphrey::http::{Request, Response, StatusCode};
use humphrey::App;

use jasondb::database::Database;
use jasondb::prelude::*;
use jasondb::JasonDB;

use std::{error::Error, sync::Arc};

fn main() -> Result<(), Box<dyn Error>> {
    // Open the database or create it if it doesn't exist.
    // This automatically starts a background thread to synchronize the database to disk.
    let database = JasonDB::open("database.jdb").unwrap_or_else(new_db);

    // Create an app with the database as the state.
    let app: App<JasonDB> = App::new_with_config(32, database)
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
fn home(request: Request, db: Arc<JasonDB>) -> Response {
    let messages = collection!(db.read(), "messages") // Use the `collection!` macro to get a collection from the database.
        .list() // List the documents in the collection.
        .iter() // Iterate over the documents, using `fold` to join together the document values.
        .fold(String::new(), |mut acc, message| {
            acc.push_str(&format!("<li>{}</li>", message.json)); // WARNING: XSS vulnerability!
            acc
        });

    // Render the template with the messages.
    let html = include_str!("../static/index.html").replace("{messages}", &messages);

    Response::new(StatusCode::OK, html, &request)
}

/// The handler for the API endpoint to add a message to the database.
fn post_message(request: Request, db: Arc<JasonDB>) -> Response {
    // If the request is not a POST request, return a 405 Method Not Allowed response.
    if request.method != Method::Post {
        return Response::new(
            StatusCode::MethodNotAllowed,
            "405 Method Not Allowed",
            &request,
        );
    }

    // Get the message from the request body.
    if let Some(body) = &request.content {
        let message = String::from_utf8(body.clone()).unwrap();

        // Use the `push!` macro of JasonDB to add the message to the `messages` collection of the database.
        push!(db.write(), "messages", message);
    }

    // Return a 200 OK response.
    Response::new(StatusCode::OK, b"OK", &request)
}

/// Create a new database, automatically starting the background thread to synchronize the database to disk.
fn new_db(_: Box<dyn Error>) -> JasonDB {
    // Create the database and the `messages` collection.
    let mut db = Database::new("database.jdb");
    db.create_collection("messages").unwrap();

    // Initialise the JasonDB instance with the pre-existing database.
    JasonDB::init(db, "database.jdb")
}

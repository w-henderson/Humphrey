use humphrey::http::{Request, Response, StatusCode};
use humphrey::App;
use std::{error::Error, sync::Arc};

#[derive(Default)]
struct AppState;

fn main() -> Result<(), Box<dyn Error>> {
    let app: App<AppState> = App::new()
        .with_route("/", home)
        .with_route("/wildcard/*", wildcard);

    app.run("127.0.0.1:80")?;

    Ok(())
}

fn home(request: &Request, _: Arc<AppState>) -> Response {
    let html = include_str!("index.html");

    Response::new(StatusCode::OK)
        .with_bytes(html.as_bytes().to_vec())
        .with_request_compatibility(request)
        .with_generated_headers()
}

fn wildcard(request: &Request, _: Arc<AppState>) -> Response {
    let wildcard_path = request
        .uri // get the URI of the request
        .strip_prefix("/wildcard/") // remove the initial slash
        .unwrap(); // unwrap from the option

    let html = format!(
        "<html><body><h1>Wildcard Path: {}</h1></body></html>",
        wildcard_path
    );

    Response::new(StatusCode::OK)
        .with_bytes(html.as_bytes().to_vec())
        .with_request_compatibility(request)
        .with_generated_headers()
}

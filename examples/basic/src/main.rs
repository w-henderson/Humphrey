use humphrey::http::{Request, Response, StatusCode};
use humphrey::App;
use std::sync::Arc;

#[derive(Default)]
struct AppState;

fn main() {
    let app: App<AppState> = App::new()
        .with_route("/", home)
        .with_route("/contact", contact);
    app.run(&("127.0.0.1:80".parse().unwrap())).unwrap();
}

fn home(request: &Request, _: Arc<AppState>) -> Response {
    Response::new(StatusCode::OK)
        .with_bytes(b"<html><body><h1>Home</h1></body></html>".to_vec())
        .with_request_compatibility(request)
        .with_generated_headers()
}

fn contact(request: &Request, _: Arc<AppState>) -> Response {
    Response::new(StatusCode::OK)
        .with_bytes(b"<html><body><h1>Contact</h1></body></html>".to_vec())
        .with_request_compatibility(request)
        .with_generated_headers()
}

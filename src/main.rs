use humphrey::http::{Request, Response, StatusCode};
use humphrey::App;

fn main() {
    let app = App::new()
        .with_route("/", home)
        .with_route("/contact", contact);
    app.run(&("127.0.0.1:80".parse().unwrap())).unwrap();
}

fn home(request: &Request) -> Response {
    Response::new(StatusCode::OK)
        .with_bytes(b"<html><body><h1>Home</h1></body></html>".to_vec())
        .with_request_compatibility(request)
        .with_generated_headers()
}

fn contact(request: &Request) -> Response {
    Response::new(StatusCode::OK)
        .with_bytes(b"<html><body><h1>Contact</h1></body></html>".to_vec())
        .with_request_compatibility(request)
        .with_generated_headers()
}

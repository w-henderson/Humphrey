use humphrey::http::{Request, Response, StatusCode};
use humphrey::App;
use std::{error::Error, sync::Arc};

fn main() -> Result<(), Box<dyn Error>> {
    let app = App::new()
        .with_route("/", home)
        .with_route("/contact", contact);
    app.run("0.0.0.0:80")?;

    Ok(())
}

fn home(request: Request, _: Arc<()>) -> Response {
    Response::new(StatusCode::OK, b"<html><body><h1>Home</h1></body></html>", &request)
}

fn contact(request: Request, _: Arc<()>) -> Response {
    Response::new(StatusCode::OK, b"<html><body><h1>Contact</h1></body></html>", &request)
}
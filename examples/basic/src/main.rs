use humphrey::http::{Request, Response, StatusCode};
use humphrey::App;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let app: App<()> = App::new()
        .with_stateless_route("/", home)
        .with_stateless_route("/contact", contact)
        .with_stateless_route("/*", generic);
    app.run("0.0.0.0:80")?;

    Ok(())
}

fn home(_: Request) -> Response {
    Response::new(StatusCode::OK, "<html><body><h1>Home</h1></body></html>")
}

fn contact(_: Request) -> Response {
    Response::new(StatusCode::OK, "<html><body><h1>Contact</h1></body></html>")
}

fn generic(request: Request) -> Response {
    let html = format!(
        "<html><body><h1>You just requested {}.</h1></body></html>",
        request.uri
    );

    Response::new(StatusCode::OK, html)
}

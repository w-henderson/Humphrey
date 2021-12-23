use humphrey::http::{Request, Response, StatusCode};
use humphrey::App;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let app: App<()> = App::new()
        .with_stateless_route("/", home)
        .with_stateless_route("/wildcard/*", wildcard);

    app.run("127.0.0.1:80")?;

    Ok(())
}

fn home(_: Request) -> Response {
    let html = include_str!("index.html");

    Response::new(StatusCode::OK, html)
}

fn wildcard(request: Request) -> Response {
    let wildcard_path = request
        .uri // get the URI of the request
        .strip_prefix("/wildcard/") // remove the initial slash
        .unwrap(); // unwrap from the option

    let html = format!(
        "<html><body><h1>Wildcard Path: {}</h1></body></html>",
        wildcard_path
    );

    Response::new(StatusCode::OK, html)
}

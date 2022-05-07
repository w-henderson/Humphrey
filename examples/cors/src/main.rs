use humphrey::http::{Request, Response, StatusCode};
use humphrey::App;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let app: App<()> = App::new().with_stateless_route("/", home).with_cors(false);

    app.run("0.0.0.0:80")?;

    Ok(())
}

fn home(_: Request) -> Response {
    Response::new(StatusCode::OK, "OK")
}

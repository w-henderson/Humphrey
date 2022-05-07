use humphrey::http::cors::Cors;
use humphrey::http::{Request, Response, StatusCode};
use humphrey::{App, SubApp};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let cors_enabled_subapp: SubApp<()> = SubApp::new()
        .with_stateless_route("/", no_cors_route)
        .with_stateless_route("/cors", cors_route)
        .with_cors(Cors::wildcard()); // enable CORS for every route of this subapp

    let app: App<()> = App::new()
        .with_host("cors.localhost", cors_enabled_subapp)
        .with_stateless_route("/", no_cors_route)
        .with_stateless_route("/cors", cors_route)
        .with_cors_config("/cors", Cors::wildcard()); // enable CORS just for the /cors route

    // In this example, CORS is enabled for the /cors route normally, but not for the / route.
    // If you access the application from `cors.localhost`, every route enables CORS.

    app.run("0.0.0.0:80")?;

    Ok(())
}

fn no_cors_route(_: Request) -> Response {
    Response::new(StatusCode::OK, "This route does not have CORS enabled")
}

fn cors_route(_: Request) -> Response {
    Response::new(StatusCode::OK, "This route has CORS enabled")
}

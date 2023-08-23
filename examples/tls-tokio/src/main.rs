//! This example requires you to provide a certificate in `keys/localhost.pem` and a key in `keys/localhost-key.pem`.
//! Follow the steps in the TLS guide [here](https://humphrey.whenderson.dev/core/https).

use humphrey::http::{Request, Response, StatusCode};
use humphrey::App;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app: App<()> = App::new()
        .with_stateless_route("/", home)
        .with_cert("keys/localhost.pem", "keys/localhost-key.pem")
        .with_forced_https(true);

    app.run_tls("0.0.0.0:443").await?;

    Ok(())
}

async fn home(_: Request) -> Response {
    Response::new(
        StatusCode::OK,
        "<html><body><h1>This is served over HTTPS!</h1></body></html>",
    )
}

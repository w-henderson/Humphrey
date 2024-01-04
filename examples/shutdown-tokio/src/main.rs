use humphrey::http::{Request, Response, StatusCode};
use humphrey::App;
use tokio_util::sync::CancellationToken;

use std::error::Error;
use std::thread::{sleep, spawn};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cancel = CancellationToken::new();
    let app: App<()> = App::new()
        .with_shutdown(cancel.clone())
        .with_stateless_route("/", hello);

    // Shutdown the main app after 5 seconds
    spawn(move || {
        sleep(Duration::from_secs(5));
        cancel.cancel();
    });

    // Returns after shutdown signal
    app.run("0.0.0.0:8080").await?;

    Ok(())
}

async fn hello(_: Request) -> Response {
    Response::new(StatusCode::OK, "Hello, world! - tokio")
}

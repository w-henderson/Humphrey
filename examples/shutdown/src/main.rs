use humphrey::http::{Response, StatusCode};
use humphrey::App;

use std::error::Error;
use std::sync::mpsc;
use std::thread::{sleep, spawn};
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    let (shutdown_app, app_rx) = mpsc::sync_channel(0);

    let app: App<()> = App::new()
        .with_shutdown(app_rx)
        .with_stateless_route("/hello", |_| Response::new(StatusCode::OK, "Hello world!"));

    // Shutdown the main app after 5 seconds
    let t = spawn(move || {
        sleep(Duration::from_secs(5));
        let _ = shutdown_app.send(());
    });

    // Returns after shutdown signal
    app.run("0.0.0.0:8080").unwrap();

    // Wait for thread to fully finish. Unneeded but placed here for full memory tests.
    t.join().unwrap();

    Ok(())
}

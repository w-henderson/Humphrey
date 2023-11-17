use humphrey::http::{Request, Response, StatusCode};
use humphrey::App;
use std::error::Error;

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

fn main() -> Result<(), Box<dyn Error>> {
    let shutdown = Arc::new(AtomicBool::new(false));
    let app: App<()> = App::new()
        .with_shutdown(shutdown.clone())
        .with_stateless_route("/", home)
        .with_stateless_route("/contact", contact)
        .with_stateless_route("/*", generic);
    
    std::thread::spawn(move || {
        if let Err(err) = app.run("0.0.0.0:80") { println!("{err}")}
        println!("app done");
    });

    std::thread::sleep(std::time::Duration::from_secs(5));
    shutdown.store(true, Ordering::Relaxed);
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

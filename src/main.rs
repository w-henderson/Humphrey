use humphrey::http::headers::ResponseHeader;
use humphrey::http::mime::MimeType;
use humphrey::http::{Request, Response, StatusCode};
use humphrey::route::try_open_path;
use humphrey::App;

mod config;
use config::Config;

use std::io::Read;
use std::sync::Arc;

fn main() {
    if let Ok(config) = config::load_config() {
        let addr = format!("{}:{}", &config.address, &config.port);
        let app: App<Config> = App::new().with_state(config).with_route("/*", file_handler);

        if let Err(_) = app.run(addr) {
            println!("An error occurred");
        };
    } else {
        println!("An error occurred loading the configuration.");
    }
}

/// Request handler for every request.
/// Attempts to open a given file relative to the binary and returns error 404 if not found.
fn file_handler(request: &Request, config: Arc<Config>) -> Response {
    let full_path = format!("{}{}", config.directory, request.uri);

    if let Some(mut located) = try_open_path(&full_path) {
        if located.was_redirected && request.uri.chars().last() != Some('/') {
            return Response::new(StatusCode::MovedPermanently)
                .with_header(ResponseHeader::Location, format!("{}/", &request.uri))
                .with_request_compatibility(request)
                .with_generated_headers();
        }

        let file_extension = if located.was_redirected {
            "html"
        } else {
            request.uri.split(".").last().unwrap_or("")
        };
        let mime_type = MimeType::from_extension(file_extension);
        let mut contents: Vec<u8> = Vec::new();
        located.file.read_to_end(&mut contents).unwrap();

        Response::new(StatusCode::OK)
            .with_header(ResponseHeader::ContentType, mime_type.into())
            .with_bytes(contents)
            .with_request_compatibility(request)
            .with_generated_headers()
    } else {
        Response::new(StatusCode::NotFound)
            .with_header(ResponseHeader::ContentType, "text/html".into())
            .with_bytes(b"<h1>404 Not Found</h1>".to_vec())
            .with_request_compatibility(request)
            .with_generated_headers()
    }
}

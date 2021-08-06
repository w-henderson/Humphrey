use humphrey::http::headers::ResponseHeader;
use humphrey::http::mime::MimeType;
use humphrey::http::{Request, Response, StatusCode};
use humphrey::route::try_open_path;
use humphrey::App;
use std::io::Read;
use std::sync::Arc;

#[derive(Default)]
struct AppConfig {
    directory_root: String,
}

fn main() {
    let app_config = AppConfig::default();

    let app: App<AppConfig> = App::new()
        .with_state(app_config)
        .with_route("/*", file_handler);

    app.run(&("0.0.0.0:80".parse().unwrap())).unwrap();
}

/// Request handler for every request.
/// Attempts to open a given file relative to the binary and returns error 404 if not found.
fn file_handler(request: &Request, config: Arc<AppConfig>) -> Response {
    let full_path = format!("{}{}", config.directory_root, request.uri);

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

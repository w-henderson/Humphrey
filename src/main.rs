use humphrey::http::headers::ResponseHeader;
use humphrey::http::mime::MimeType;
use humphrey::http::{Request, Response, StatusCode};
use humphrey::App;
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct AppState;

fn main() {
    let app: App<AppState> = App::new().with_route("/*", file);
    app.run(&("0.0.0.0:80".parse().unwrap())).unwrap();
}

/// Request handler for every request.
/// Attempts to open a given file relative to the binary and returns error 404 if not found.
fn file(request: &Request, _: Arc<Mutex<AppState>>) -> Response {
    let path = if request.uri.path.len() == 0 {
        "index.html".to_string()
    } else {
        request.uri.path.join("/")
    };

    if let Ok(mut file) = File::open(&path) {
        let file_extension = path.split(".").last().unwrap_or("");
        let mime_type = MimeType::from_extension(file_extension);
        let mut contents: Vec<u8> = Vec::new();
        file.read_to_end(&mut contents).unwrap();

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

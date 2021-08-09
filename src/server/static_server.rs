use humphrey::http::headers::ResponseHeader;
use humphrey::http::mime::MimeType;
use humphrey::http::{Request, Response, StatusCode};
use humphrey::App;

use crate::cache::Cache;
use crate::config::Config;
use crate::route::try_open_path;

use std::io::Read;
use std::sync::{Arc, RwLock};

#[derive(Default)]
struct AppState {
    directory: String,
    cache: RwLock<Cache>,
}

impl From<&Config> for AppState {
    fn from(config: &Config) -> Self {
        Self {
            directory: config.directory.as_ref().unwrap().clone(),
            cache: RwLock::new(Cache::from(config)),
        }
    }
}

/// Main function for the static server.
pub fn main(config: Config) {
    let app: App<AppState> = App::new()
        .with_state(AppState::from(&config))
        .with_route("/*", file_handler);

    let addr = format!("{}:{}", config.address, config.port);

    app.run(addr).unwrap();
}

/// Request handler for every request.
/// Attempts to open a given file relative to the binary and returns error 404 if not found.
fn file_handler(request: &Request, state: Arc<AppState>) -> Response {
    let full_path = format!("{}{}", state.directory, request.uri);

    let cache = state.cache.read().unwrap();
    let cache_limit = cache.cache_limit;

    if cache_limit > 0 {
        if let Some(cached) = cache.get(&full_path) {
            return Response::new(StatusCode::OK)
                .with_header(ResponseHeader::ContentType, cached.mime_type.into())
                .with_bytes(cached.data.clone())
                .with_request_compatibility(request)
                .with_generated_headers();
        }
    }

    drop(cache);

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

        if cache_limit >= contents.len() {
            let mut cache = state.cache.write().unwrap();
            cache.set(&full_path, contents.clone(), mime_type);
        }

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

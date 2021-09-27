use crate::route::try_find_path;
use crate::server::server::AppState;

use humphrey::http::headers::ResponseHeader;
use humphrey::http::mime::MimeType;
use humphrey::http::{Request, Response, StatusCode};

use std::fs::File;
use std::io::Read;
use std::sync::Arc;

/// Request handler for every request.
/// Attempts to open a given file relative to the binary and returns error 404 if not found.
pub fn file_handler(request: Request, state: Arc<AppState>, directory: &str) -> Response {
    // Return error 403 if the address was blacklisted
    if state
        .config
        .blacklist
        .list
        .contains(&request.address.origin_addr.to_string())
    {
        state.logger.warn(&format!(
            "{}: Blacklisted IP attempted to request {}",
            request.address, request.uri
        ));
        return Response::new(StatusCode::Forbidden)
            .with_header(ResponseHeader::ContentType, "text/html".into())
            .with_bytes(b"<h1>403 Forbidden</h1>".to_vec())
            .with_request_compatibility(&request)
            .with_generated_headers();
    }

    if state.config.cache.size_limit > 0 {
        let cache = state.cache.read().unwrap();
        if let Some(cached) = cache.get(&request.uri) {
            state.logger.info(&format!(
                "{}: 200 OK (cached) {}",
                request.address, request.uri
            ));
            return Response::new(StatusCode::OK)
                .with_header(ResponseHeader::ContentType, cached.mime_type.into())
                .with_bytes(cached.data.clone())
                .with_request_compatibility(&request)
                .with_generated_headers();
        }
        drop(cache);
    }

    if let Some(located) = try_find_path(directory, &request.uri) {
        if located.was_redirected && request.uri.chars().last() != Some('/') && request.uri != "" {
            state.logger.info(&format!(
                "{}: 301 Moved Permanently {}",
                request.address, request.uri
            ));
            return Response::new(StatusCode::MovedPermanently)
                .with_header(ResponseHeader::Location, format!("{}/", &request.uri))
                .with_request_compatibility(&request)
                .with_generated_headers();
        }

        let file_extension = located
            .path
            .extension()
            .map(|s| s.to_str().unwrap())
            .unwrap_or("");

        let mime_type = MimeType::from_extension(file_extension);
        let mut contents: Vec<u8> = Vec::new();

        let mut file = File::open(located.path).unwrap();
        file.read_to_end(&mut contents).unwrap();

        if state.config.cache.size_limit >= contents.len() {
            let mut cache = state.cache.write().unwrap();
            cache.set(&request.uri, contents.clone(), mime_type);
            state.logger.debug(&format!("Cached route {}", request.uri));
        } else if state.config.cache.size_limit > 0 {
            state
                .logger
                .warn(&format!("Couldn't cache, cache too small {}", request.uri));
        }

        state
            .logger
            .info(&format!("{}: 200 OK {}", request.address, request.uri));
        Response::new(StatusCode::OK)
            .with_header(ResponseHeader::ContentType, mime_type.into())
            .with_bytes(contents)
            .with_request_compatibility(&request)
            .with_generated_headers()
    } else {
        state.logger.warn(&format!(
            "{}: 404 Not Found {}",
            request.address, request.uri
        ));
        not_found(&request)
    }
}

pub fn not_found(request: &Request) -> Response {
    Response::new(StatusCode::NotFound)
        .with_header(ResponseHeader::ContentType, "text/html".into())
        .with_bytes(b"<h1>404 Not Found</h1>".to_vec())
        .with_request_compatibility(request)
        .with_generated_headers()
}

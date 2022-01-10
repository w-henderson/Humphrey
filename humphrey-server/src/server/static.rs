//! Provides functionality for serving static content.

use crate::server::server::AppState;

use humphrey::http::headers::ResponseHeader;
use humphrey::http::mime::MimeType;
use humphrey::http::{Request, Response, StatusCode};
use humphrey::route::{try_find_path, LocatedPath};

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;

const INDEX_FILES: [&str; 2] = ["index.html", "index.htm"];

/// Request handler for files.
pub fn file_handler(request: Request, state: Arc<AppState>, file: &str, host: usize) -> Response {
    if let Some(response) = blacklist_check(&request, state.clone()) {
        return response;
    }

    if let Some(response) = cache_check(&request, state.clone(), host) {
        return response;
    }

    inner_file_handler(request, state, file.into(), host)
}

/// Request handler for directories.
/// Attempts to open a given file relative to the binary and returns error 404 if not found.
pub fn directory_handler(
    request: Request,
    state: Arc<AppState>,
    directory: &str,
    matches: &str,
    host: usize,
) -> Response {
    if let Some(response) = blacklist_check(&request, state.clone()) {
        return response;
    }

    if let Some(response) = cache_check(&request, state.clone(), host) {
        return response;
    }

    let mut simplified_uri = request.uri.clone();

    for ch in matches.chars() {
        if ch != '*' {
            simplified_uri.remove(0);
        } else {
            break;
        }
    }

    if let Some(located) = try_find_path(directory, &simplified_uri, &INDEX_FILES) {
        match located {
            LocatedPath::Directory => {
                state.logger.info(&format!(
                    "{}: 301 Moved Permanently {}",
                    request.address, request.uri
                ));
                Response::empty(StatusCode::MovedPermanently)
                    .with_header(ResponseHeader::Location, format!("{}/", &request.uri))
            }
            LocatedPath::File(path) => inner_file_handler(request, state, path, host),
        }
    } else {
        state.logger.warn(&format!(
            "{}: 404 Not Found {}",
            request.address, request.uri
        ));
        not_found()
    }
}

/// Request handler for redirects.
pub fn redirect_handler(request: Request, state: Arc<AppState>, target: &str) -> Response {
    if let Some(response) = blacklist_check(&request, state.clone()) {
        return response;
    }

    state.logger.info(&format!(
        "{}: 301 Moved Permanently {}",
        request.address, request.uri
    ));
    Response::empty(StatusCode::MovedPermanently)
        .with_header(ResponseHeader::Location, target.into())
}

fn inner_file_handler(
    request: Request,
    state: Arc<AppState>,
    path: PathBuf,
    host: usize,
) -> Response {
    let file_extension = path.extension().map(|s| s.to_str().unwrap()).unwrap_or("");

    let mime_type = MimeType::from_extension(file_extension);
    let mut contents: Vec<u8> = Vec::new();

    let mut file = File::open(path).unwrap();
    file.read_to_end(&mut contents).unwrap();

    if state.config.cache.size_limit >= contents.len() {
        let mut cache = state.cache.write().unwrap();
        cache.set(&request.uri, host, contents.clone(), mime_type);
        state.logger.debug(&format!("Cached route {}", request.uri));
    } else if state.config.cache.size_limit > 0 {
        state
            .logger
            .warn(&format!("Couldn't cache, cache too small {}", request.uri));
    }

    state
        .logger
        .info(&format!("{}: 200 OK {}", request.address, request.uri));
    Response::empty(StatusCode::OK)
        .with_header(ResponseHeader::ContentType, mime_type.into())
        .with_bytes(contents)
}

fn blacklist_check(request: &Request, state: Arc<AppState>) -> Option<Response> {
    // Return error 403 if the address was blacklisted
    if state
        .config
        .blacklist
        .list
        .contains(&request.address.origin_addr)
    {
        state.logger.warn(&format!(
            "{}: Blacklisted IP attempted to request {}",
            request.address, request.uri
        ));
        return Some(
            Response::empty(StatusCode::Forbidden)
                .with_header(ResponseHeader::ContentType, "text/html".into())
                .with_bytes(b"<h1>403 Forbidden</h1>"),
        );
    }

    None
}

fn cache_check(request: &Request, state: Arc<AppState>, host: usize) -> Option<Response> {
    if state.config.cache.size_limit > 0 {
        let cache = state.cache.read().unwrap();
        if let Some(cached) = cache.get(&request.uri, host) {
            state.logger.info(&format!(
                "{}: 200 OK (cached) {}",
                request.address, request.uri
            ));
            return Some(
                Response::empty(StatusCode::OK)
                    .with_header(ResponseHeader::ContentType, cached.mime_type.into())
                    .with_bytes(cached.data.clone()),
            );
        }
        drop(cache);
    }

    None
}

/// Generates a 404 response.
pub fn not_found() -> Response {
    Response::empty(StatusCode::NotFound)
        .with_header(ResponseHeader::ContentType, "text/html".into())
        .with_bytes(b"<h1>404 Not Found</h1>")
}

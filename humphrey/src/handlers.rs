use crate::app::error_handler;
use crate::http::headers::ResponseHeader;
use crate::http::{Request, Response, StatusCode};
use crate::route::{try_find_path, LocatedPath};

use std::fs::File;
use std::io::Read;
use std::sync::Arc;

const INDEX_FILES: [&str; 2] = ["index.html", "index.htm"];

/// Serve the specified file, or a default error 404 if not found.
pub fn serve_file<T>(file_path: &'static str) -> impl Fn(Request, Arc<T>) -> Response {
    move |request: Request, _| {
        if let Ok(mut file) = File::open(file_path) {
            let mut buf = Vec::new();
            if file.read_to_end(&mut buf).is_ok() {
                return Response::new(StatusCode::OK, buf, &request);
            }
        }

        error_handler(Some(request), StatusCode::NotFound)
    }
}

/// Treat the request URI as a file path relative to the given directory and serve files from there.
///
/// ## Examples
/// - directory path of `.` will serve files relative to the current directory
/// - directory path of `./static` will serve files from the static directory but with their whole URI,
///     for example a request to `/images/ferris.png` will map to the file `./static/images/ferris.png`.
///
/// This is **not** equivalent to `serve_dir`, as `serve_dir` respects index files within nested directories.
pub fn serve_as_file_path<T>(directory_path: &'static str) -> impl Fn(Request, Arc<T>) -> Response {
    move |request: Request, _| {
        let directory_path = directory_path.strip_suffix('/').unwrap_or(directory_path);
        let file_path = request.uri.strip_prefix('/').unwrap_or(&request.uri);
        let path = format!("{}/{}", directory_path, file_path);

        if let Ok(mut file) = File::open(path) {
            let mut buf = Vec::new();
            if file.read_to_end(&mut buf).is_ok() {
                return Response::new(StatusCode::OK, buf, &request);
            }
        }

        error_handler(Some(request), StatusCode::NotFound)
    }
}

/// Serves a directory of files.
///
/// Respects index files with the following rules:
///   - requests to `/directory` will return either the file `directory`, 301 redirect to `/directory/` if it is a directory, or return 404
///   - requests to `/directory/` will return either the file `/directory/index.html` or `/directory/index.htm`, or return 404
pub fn serve_dir<T>(directory_path: &'static str) -> impl Fn(Request, Arc<T>, &str) -> Response {
    move |request: Request, _, route| {
        let route_without_wildcard = route.strip_suffix('*').unwrap_or(route);
        let uri_without_route = request
            .uri
            .strip_prefix(route_without_wildcard)
            .unwrap_or(&request.uri);

        let located = try_find_path(directory_path, uri_without_route, &INDEX_FILES);

        if let Some(located) = located {
            match located {
                LocatedPath::Directory => Response::empty(StatusCode::MovedPermanently)
                    .with_header(ResponseHeader::Location, format!("{}/", &request.uri))
                    .with_request_compatibility(&request)
                    .with_generated_headers(),
                LocatedPath::File(path) => {
                    if let Ok(mut file) = File::open(path) {
                        let mut buf = Vec::new();
                        if file.read_to_end(&mut buf).is_ok() {
                            return Response::new(StatusCode::OK, buf, &request);
                        }
                    }

                    error_handler(Some(request), StatusCode::InternalError)
                }
            }
        } else {
            error_handler(Some(request), StatusCode::NotFound)
        }
    }
}

/// Redirects requests to the given location with status code 301.
pub fn redirect<T>(location: &'static str) -> impl Fn(Request, Arc<T>) -> Response {
    move |request: Request, _| Response::redirect(location, &request)
}

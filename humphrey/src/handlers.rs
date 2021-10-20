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
/// This is **not** equivalent to `serve_dir` with a strip prefix value of `""`, as this does not respect
///   index files within nested directories.
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

/// Serves a directory of files, stripping the given prefix from the request URI before concatenating it with
///   the given directory path.
///
/// Respects index files with the following rules:
///   - requests to `/directory` will return either the file `directory`, 301 redirect to `/directory/` if it is a directory, or return 404
///   - requests to `/directory/` will return either the file `/directory/index.html` or `/directory/index.htm`, or return 404
///
/// ## Example
/// For example, if you want to serve files from a `./static/images` directory to the `/img` route of your server,
///   you would use `serve_dir` with a directory path of `./static/images` and a strip prefix value of `/img/` at
///   a route called `/img/*`. This would remove the `/img/` prefix from the request URI before concatenating it
///   with the directory path `./static/images`.
pub fn serve_dir<T>(
    directory_path: &'static str,
    strip_prefix: &'static str,
) -> impl Fn(Request, Arc<T>) -> Response {
    move |request: Request, _| {
        let strip_prefix_without_slash = strip_prefix.strip_prefix('/').unwrap_or(&request.uri);

        let stripped_prefix = request.uri[1..]
            .strip_prefix(strip_prefix_without_slash)
            .unwrap_or(&request.uri);

        let located = try_find_path(directory_path, stripped_prefix, &INDEX_FILES);

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

//! Provides a number of useful handlers for Humphrey apps.

// FIXME: all the structs and stuff in this file are a workaround for async closures not being stable yet.
// When they are stabilised, this code will look a lot nicer.

use crate::app::{error_handler, PathAwareRequestHandler, RequestHandler};
use crate::http::headers::HeaderType;
use crate::http::mime::MimeType;
use crate::http::{Request, Response, StatusCode};
use crate::route::{try_find_path, LocatedPath};

use tokio::fs::File;
use tokio::io::AsyncReadExt;

use futures::Future;

use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

const INDEX_FILES: [&str; 2] = ["index.html", "index.htm"];

/// Serve the specified file, or a default error 404 if not found.
pub fn serve_file<S>(file_path: &'static str) -> impl RequestHandler<S> {
    let path_buf = PathBuf::from(file_path);

    FileServer { path_buf }
}

struct FileServer {
    path_buf: PathBuf,
}

impl<S> RequestHandler<S> for FileServer {
    fn serve(&self, _: Request, _: Arc<S>) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        let path_buf = self.path_buf.clone();

        Box::pin(async move {
            if let Ok(mut file) = File::open(&path_buf).await {
                let mut buf = Vec::new();
                if file.read_to_end(&mut buf).await.is_ok() {
                    return if let Some(extension) = &path_buf.extension() {
                        Response::new(StatusCode::OK, buf).with_header(
                            HeaderType::ContentType,
                            MimeType::from_extension(extension.to_str().unwrap()).to_string(),
                        )
                    } else {
                        Response::new(StatusCode::OK, buf)
                    };
                }
            }

            error_handler(StatusCode::NotFound)
        })
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
pub fn serve_as_file_path<S>(directory_path: &'static str) -> impl RequestHandler<S> {
    let directory_path = PathBuf::from(directory_path.strip_suffix('/').unwrap_or(directory_path));

    FilePathServer { directory_path }
}

struct FilePathServer {
    directory_path: PathBuf,
}

impl<S> RequestHandler<S> for FilePathServer {
    fn serve(&self, request: Request, _: Arc<S>) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        let directory_path = self.directory_path.clone();

        Box::pin(async move {
            let file_path = request.uri.strip_prefix('/').unwrap_or(&request.uri);
            let path = format!("{}/{}", directory_path.to_str().unwrap(), file_path);

            let path_buf = PathBuf::from(path);

            if let Ok(mut file) = File::open(&path_buf).await {
                let mut buf = Vec::new();
                if file.read_to_end(&mut buf).await.is_ok() {
                    return if let Some(extension) = path_buf.extension() {
                        Response::new(StatusCode::OK, buf).with_header(
                            HeaderType::ContentType,
                            MimeType::from_extension(extension.to_str().unwrap()).to_string(),
                        )
                    } else {
                        Response::new(StatusCode::OK, buf)
                    };
                }
            }

            error_handler(StatusCode::NotFound)
        })
    }
}

/// Serves a directory of files.
///
/// Respects index files with the following rules:
///   - requests to `/directory` will return either the file `directory`, 301 redirect to `/directory/` if it is a directory, or return 404
///   - requests to `/directory/` will return either the file `/directory/index.html` or `/directory/index.htm`, or return 404
pub fn serve_dir<S>(directory_path: &'static str) -> impl PathAwareRequestHandler<S> {
    DirServer { directory_path }
}

struct DirServer {
    directory_path: &'static str,
}

impl<S> PathAwareRequestHandler<S> for DirServer {
    fn serve(
        &self,
        request: Request,
        _: Arc<S>,
        route: &'static str,
    ) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        let directory_path = self.directory_path;

        Box::pin(async move {
            let route_without_wildcard = route.strip_suffix('*').unwrap_or(route);
            let uri_without_route = request
                .uri
                .strip_prefix(route_without_wildcard)
                .unwrap_or(&request.uri);

            let located = try_find_path(directory_path, uri_without_route, &INDEX_FILES);

            if let Some(located) = located {
                match located {
                    LocatedPath::Directory => Response::empty(StatusCode::MovedPermanently)
                        .with_header(HeaderType::Location, format!("{}/", &request.uri)),
                    LocatedPath::File(path) => {
                        if let Ok(mut file) = File::open(&path).await {
                            let mut buf = Vec::new();
                            if file.read_to_end(&mut buf).await.is_ok() {
                                return if let Some(extension) = path.extension() {
                                    Response::new(StatusCode::OK, buf).with_header(
                                        HeaderType::ContentType,
                                        MimeType::from_extension(extension.to_str().unwrap())
                                            .to_string(),
                                    )
                                } else {
                                    Response::new(StatusCode::OK, buf)
                                };
                            }
                        }

                        error_handler(StatusCode::InternalError)
                    }
                }
            } else {
                error_handler(StatusCode::NotFound)
            }
        })
    }
}

/// Redirects requests to the given location with status code 301.
pub fn redirect<S>(location: &'static str) -> impl RequestHandler<S> {
    RedirectServer { location }
}

struct RedirectServer {
    location: &'static str,
}

impl<S> RequestHandler<S> for RedirectServer {
    fn serve(&self, _: Request, _: Arc<S>) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        let location = self.location;

        Box::pin(async move { Response::redirect(location) })
    }
}

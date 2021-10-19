use crate::app::RequestHandler;
use crate::krauss;

use std::fs::metadata;
use std::path::PathBuf;

/// Encapsulates a route and its handler.
pub struct RouteHandler<State> {
    pub route: String,
    pub handler: RequestHandler<State>,
}

/// An object that can represent a route, currently only `String`.
pub trait Route {
    fn route_matches(&self, route: &str) -> bool;
}

impl Route for String {
    /// Checks whether this route matches the given one, respecting its own wildcards only.
    /// For example, `/blog/*` will match `/blog/my-first-post` but not the other way around.
    fn route_matches(&self, route: &str) -> bool {
        krauss::wildcard_match(self, route)
    }
}

/// A located file or directory path.
pub enum LocatedPath {
    Directory,
    File(PathBuf),
}

/// Attemps to find a given path.
/// If the path itself is not found, attemps to find index files within it.
/// If these are not found, returns `None`.
pub fn try_find_path(
    directory: &str,
    request_path: &str,
    index_files: &[&str],
) -> Option<LocatedPath> {
    // Avoid path traversal exploits
    if request_path.contains("..") || request_path.contains(':') {
        return None;
    }

    let request_path = request_path.trim_start_matches('/');
    let directory = directory.trim_end_matches('/');

    if request_path.ends_with('/') || request_path.is_empty() {
        for filename in index_files {
            let path = format!("{}/{}{}", directory, request_path, *filename);
            if let Ok(meta) = metadata(&path) {
                if meta.is_file() {
                    return Some(LocatedPath::File(
                        PathBuf::from(path).canonicalize().unwrap(),
                    ));
                }
            }
        }
    } else {
        let path = format!("{}/{}", directory, request_path);

        if let Ok(meta) = metadata(&path) {
            if meta.is_file() {
                return Some(LocatedPath::File(
                    PathBuf::from(path).canonicalize().unwrap(),
                ));
            } else if meta.is_dir() {
                return Some(LocatedPath::Directory);
            }
        }
    }

    None
}

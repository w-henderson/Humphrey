use crate::app::{PathAwareRequestHandler, RequestHandler, StatelessRequestHandler};
use crate::krauss;

use std::fs::metadata;
use std::path::PathBuf;

/// Represents a sub-app to run for a specific host.
pub struct SubApp<State> {
    pub host: String,
    pub routes: Vec<RouteHandler<State>>,
}

/// Encapsulates a route and its handler.
pub struct RouteHandler<State> {
    pub route: String,
    pub handler: Box<dyn RequestHandler<State>>,
}

impl<State> Default for SubApp<State> {
    fn default() -> Self {
        SubApp {
            host: "*".to_string(),
            routes: Vec::new(),
        }
    }
}

impl<State> SubApp<State> {
    /// Create a new sub-app with no routes.
    pub fn new() -> Self {
        SubApp::default()
    }

    /// Adds a route and associated handler to the sub-app.
    /// Routes can include wildcards, for example `/blog/*`.
    ///
    /// ## Panics
    /// This function will panic if the route string cannot be converted to a `Uri` object.
    pub fn with_route<T>(mut self, route: &str, handler: T) -> Self
    where
        T: RequestHandler<State> + 'static,
    {
        self.routes.push(RouteHandler {
            route: route.parse().unwrap(),
            handler: Box::new(handler),
        });
        self
    }

    /// Adds a route and associated handler to the sub-app.
    /// Does not pass the state to the handler.
    /// Routes can include wildcards, for example `/blog/*`.
    ///
    /// If you want to access the app's state in the handler, consider using `with_route`.
    ///
    /// ## Panics
    /// This function will panic if the route string cannot be converted to a `Uri` object.
    pub fn with_stateless_route<T>(mut self, route: &str, handler: T) -> Self
    where
        T: StatelessRequestHandler<State> + 'static,
    {
        self.routes.push(RouteHandler {
            route: route.parse().unwrap(),
            handler: Box::new(move |request, _| handler(request)),
        });
        self
    }

    /// Adds a path-aware route and associated handler to the sub-app.
    /// Routes can include wildcards, for example `/blog/*`.
    /// Will also pass the route to the handler at runtime.
    ///
    /// ## Panics
    /// This function will panic if the route string cannot be converted to a `Uri` object.
    pub fn with_path_aware_route<T>(mut self, route: &'static str, handler: T) -> Self
    where
        T: PathAwareRequestHandler<State> + 'static,
    {
        self.routes.push(RouteHandler {
            route: route.parse().unwrap(),
            handler: Box::new(move |request, state| handler(request, state, route)),
        });
        self
    }
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

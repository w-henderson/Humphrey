//! Provides functionality for handling app routes.

use crate::app::{
    PathAwareRequestHandler, RequestHandler, StatelessRequestHandler, WebsocketHandler,
};
use crate::krauss;
use crate::percent::PercentDecode;

use std::fs::metadata;
use std::path::PathBuf;

/// Represents a sub-app to run for a specific host.
pub struct SubApp<State> {
    /// The host to process requests for.
    pub host: String,
    /// The routes to process requests for and their handlers.
    pub routes: Vec<RouteHandler<State>>,
    /// The routes to process WebSocket requests for and their handlers.
    pub websocket_routes: Vec<WebsocketRouteHandler<State>>,
}

/// Encapsulates a route and its handler.
pub struct RouteHandler<State> {
    /// The route that this handler will match.
    pub route: String,
    /// The handler to run when the route is matched.
    pub handler: Box<dyn RequestHandler<State>>,
}

/// Encapsulates a route and its WebSocket handler.
pub struct WebsocketRouteHandler<State> {
    /// The route that this handler will match.
    pub route: String,
    /// The handler to run when the route is matched.
    pub handler: Box<dyn WebsocketHandler<State>>,
}

impl<State> Default for SubApp<State> {
    fn default() -> Self {
        SubApp {
            host: "*".to_string(),
            routes: Vec::new(),
            websocket_routes: Vec::new(),
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
    pub fn with_route<T>(mut self, route: &str, handler: T) -> Self
    where
        T: RequestHandler<State> + 'static,
    {
        self.routes.push(RouteHandler {
            route: route.to_string(),
            handler: Box::new(handler),
        });
        self
    }

    /// Adds a route and associated handler to the sub-app.
    /// Does not pass the state to the handler.
    /// Routes can include wildcards, for example `/blog/*`.
    ///
    /// If you want to access the app's state in the handler, consider using `with_route`.
    pub fn with_stateless_route<T>(mut self, route: &str, handler: T) -> Self
    where
        T: StatelessRequestHandler<State> + 'static,
    {
        self.routes.push(RouteHandler {
            route: route.to_string(),
            handler: Box::new(move |request, _| handler(request)),
        });
        self
    }

    /// Adds a path-aware route and associated handler to the sub-app.
    /// Routes can include wildcards, for example `/blog/*`.
    /// Will also pass the route to the handler at runtime.
    pub fn with_path_aware_route<T>(mut self, route: &'static str, handler: T) -> Self
    where
        T: PathAwareRequestHandler<State> + 'static,
    {
        self.routes.push(RouteHandler {
            route: route.to_string(),
            handler: Box::new(move |request, state| handler(request, state, route)),
        });
        self
    }

    /// Adds a WebSocket route and associated handler to the sub-app.
    /// Routes can include wildcards, for example `/ws/*`.
    /// The handler is passed the stream, state, and the request which triggered its calling.
    pub fn with_websocket_route<T>(mut self, route: &str, handler: T) -> Self
    where
        T: WebsocketHandler<State> + 'static,
    {
        self.websocket_routes.push(WebsocketRouteHandler {
            route: route.to_string(),
            handler: Box::new(handler),
        });
        self
    }
}

/// An object that can represent a route, currently only `String`.
pub trait Route {
    /// Returns true if the given route matches the path.
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
    /// A directory was located.
    Directory,
    /// A file was located at the given path.
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
    let request_path = String::from_utf8(request_path.percent_decode()?).ok()?;

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

use crate::app::RequestHandler;
use crate::krauss;
use std::fs::File;

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

pub struct LocatedPath {
    pub file: File,
    pub was_redirected: bool,
}

/// Attemps to open a given path.
/// If the path itself is not found, attemps to open index files within it.
/// If these are not found, returns `None`.
pub fn try_open_path(path: &str) -> Option<LocatedPath> {
    let paths = vec![
        path[1..].to_string(),
        format!("{}/index.html", &path[1..]),
        format!("{}/index.htm", &path[1..]),
    ];

    for (index, path) in paths.iter().enumerate() {
        if let Ok(file) = File::open(path) {
            return Some(LocatedPath {
                file,
                was_redirected: index != 0,
            });
        }
    }

    None
}

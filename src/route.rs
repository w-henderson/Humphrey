use crate::app::RequestHandler;
use crate::krauss;

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

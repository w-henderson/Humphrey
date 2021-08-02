use crate::app::RequestHandler;
use std::str::FromStr;

/// Encapsulates a route and its handler.
pub struct RouteHandler<State> {
    pub route: Uri,
    pub handler: RequestHandler<State>,
}

/// Represents a URI as part of the request.
/// Contains both the path as a `Vec<String>` and the query string.
#[derive(Debug, PartialEq, Eq)]
pub struct Uri {
    pub path: Vec<String>,
    pub query: Option<String>,
}

impl Uri {
    /// Creates a new `Uri` object with the given parameters.
    pub fn new(path: Vec<String>, query: Option<String>) -> Self {
        Self { path, query }
    }

    /// Checks whether this URI matches the given one, respecting its own wildcards only.
    /// For example, `/blog/*` will match `/blog/my-first-post` but not the other way around.
    pub fn matches(&self, other: &Uri) -> bool {
        if self.path.len() == 0 {
            if other.path.len() == 0 {
                return true;
            } else {
                return false;
            }
        }

        if self.path[self.path.len() - 1] == "*" {
            &self.path[0..self.path.len() - 1] == &other.path[0..self.path.len() - 1]
        } else {
            self.path == other.path
        }
    }
}

impl FromStr for Uri {
    type Err = ();

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if string.chars().nth(0) != Some('/') {
            return Err(());
        }

        // Split the string into the path and the query string
        let split: Vec<&str> = string.splitn(2, '?').collect();

        // Extract and parse the path
        let path: Vec<String> = split[0][1..]
            .split('/') // split by slashes
            .map(|s| s.to_string()) // store on the heap
            .filter(|s| s != "") // remove first slash which caused an empty string
            .collect(); // collect into a Vec

        // Extract and parse the query string
        let query = split.iter().nth(1).map(|s| s.to_string());

        Ok(Self { path, query })
    }
}

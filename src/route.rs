use crate::app::RequestHandler;
use std::{fs::File, str::FromStr};

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
    pub trailing_slash: bool,
    pub query: Option<String>,
}

impl Uri {
    /// Creates a new `Uri` object with the given parameters.
    pub fn new(path: Vec<String>, query: Option<String>, trailing_slash: bool) -> Self {
        Self {
            path,
            query,
            trailing_slash,
        }
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

        // Store whether the string ends with a `/`
        let trailing_slash = string.chars().last() == Some('/');

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

        Ok(Self {
            path,
            query,
            trailing_slash,
        })
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
        path.to_string(),
        format!("{}/index.html", path),
        format!("{}/index.htm", path),
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

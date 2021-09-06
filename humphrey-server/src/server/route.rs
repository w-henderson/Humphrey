use std::fs::metadata;
use std::path::PathBuf;

pub struct LocatedPath {
    pub path: PathBuf,
    pub was_redirected: bool,
}

/// Attemps to find a given path.
/// If the path itself is not found, attemps to find index files within it.
/// If these are not found, returns `None`.
pub fn try_find_path(path: &str) -> Option<LocatedPath> {
    let paths = if &path.chars().nth(0) == &Some('/') {
        vec![
            path[1..].to_string(),
            format!("{}/index.html", &path[1..]),
            format!("{}/index.htm", &path[1..]),
        ]
    } else {
        vec![
            path.to_string(),
            format!("{}/index.html", &path),
            format!("{}/index.htm", &path),
        ]
    };

    for (index, path) in paths.iter().enumerate() {
        if let Ok(meta) = metadata(path) {
            if meta.is_file() {
                return Some(LocatedPath {
                    path: PathBuf::from(path).canonicalize().unwrap(),
                    was_redirected: index != 0,
                });
            }
        }
    }

    None
}

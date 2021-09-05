use std::fs::{metadata, File};

pub struct LocatedPath {
    pub file: File,
    pub was_redirected: bool,
}

/// Attemps to open a given path.
/// If the path itself is not found, attemps to open index files within it.
/// If these are not found, returns `None`.
pub fn try_open_path(path: &str) -> Option<LocatedPath> {
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
                if let Ok(file) = File::open(path) {
                    return Some(LocatedPath {
                        file,
                        was_redirected: index != 0,
                    });
                }
            }
        }
    }

    None
}

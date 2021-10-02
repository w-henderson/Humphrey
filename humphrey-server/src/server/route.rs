use std::fs::metadata;
use std::path::PathBuf;

const INDEX_FILES: [&str; 3] = ["index.html", "index.htm", "index.php"];

pub struct LocatedPath {
    pub path: PathBuf,
    pub was_redirected: bool,
}

/// Attemps to find a given path.
/// If the path itself is not found, attemps to find index files within it.
/// If these are not found, returns `None`.
pub fn try_find_path(directory: &str, request_path: &str) -> Option<LocatedPath> {
    if request_path.contains("..") || request_path.contains(':') {
        return None;
    }

    let request_path = request_path.trim_start_matches('/');
    let directory = directory.trim_end_matches('/');

    let mut paths: Vec<String> = INDEX_FILES
        .iter()
        .map(|s| format!("{}/{}/{}", directory, request_path, s))
        .collect();
    paths.insert(0, format!("{}/{}", directory, request_path));

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

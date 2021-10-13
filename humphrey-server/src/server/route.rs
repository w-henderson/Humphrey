use std::fs::metadata;
use std::path::PathBuf;

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

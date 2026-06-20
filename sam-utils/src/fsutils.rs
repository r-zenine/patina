use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;

type Result<T> = std::result::Result<T, ErrorsFS>;

/// Returns all files reachable within two levels of `path`.
///
/// Includes files directly in `path` and files inside immediate subdirectories.
/// Deeper subdirectories and their contents are silently ignored.
pub fn walk_dir(path: &Path) -> Result<Vec<PathBuf>> {
    let dir_content = std::fs::read_dir(path)?;
    let paths = dir_content.flat_map(|e| e.map(|e| e.path()));
    let mut deque = vec![];
    for content in paths {
        if content.is_dir() {
            let cur_dir = std::fs::read_dir(content.as_path())?;
            let paths = cur_dir.flat_map(|e| e.map(|e| e.path()));
            deque.extend(paths);
        } else if content.is_file() {
            deque.push(content);
        }
    }
    Ok(deque)
}

/// Expands a leading `~` / `~/` and any `$HOME` token to the user's home
/// directory. Returns the path unchanged if the home directory can't be
/// determined.
pub fn expand_home(path: &Path) -> PathBuf {
    let Some(home_dir) = dirs::home_dir() else {
        return path.to_path_buf();
    };

    // Handle a leading `~` / `~/...` component.
    let mut components = path.components();
    if let Some(std::path::Component::Normal(first)) = components.clone().next() {
        if first == "~" {
            let remainder = components.by_ref().skip(1).collect::<PathBuf>();
            return home_dir.join(remainder);
        }
    }

    // Handle a `$HOME` token anywhere in the path.
    if let (Some(path_str), Some(home_str)) = (path.to_str(), home_dir.to_str()) {
        if path_str.contains("$HOME") {
            return PathBuf::from(path_str.replace("$HOME", home_str));
        }
    }

    path.to_path_buf()
}

pub fn ensure_exists(path: PathBuf) -> Result<PathBuf> {
    if !path.exists() {
        Err(ErrorsFS::PathDoesNotExist(path))
    } else {
        Ok(path)
    }
}

pub fn ensure_is_directory(path: PathBuf) -> Result<PathBuf> {
    if !path.is_dir() {
        Err(ErrorsFS::PathNotDirectory(path))
    } else {
        Ok(path)
    }
}

pub fn ensure_is_file(path: PathBuf) -> Result<PathBuf> {
    if !path.is_file() {
        Err(ErrorsFS::PathNotFile(path))
    } else {
        Ok(path)
    }
}

pub fn ensure_sufficient_permisions(path: PathBuf) -> Result<PathBuf> {
    std::fs::metadata(path.as_path())
        .map_err(|_| ErrorsFS::PathInsufficientPermission(path.clone()))
        .map(|_| path)
}

#[derive(Debug, Error)]
pub enum ErrorsFS {
    #[error("provided path {0} is not a directory")]
    PathNotDirectory(PathBuf),
    #[error("provided path {0} is not a file")]
    PathNotFile(PathBuf),
    #[error("provided path {0} does not exist")]
    PathDoesNotExist(PathBuf),
    #[error("insufficient permission for provided path {0}")]
    PathInsufficientPermission(PathBuf),
    #[error("got an unexpected error {0}")]
    UnexpectedIOError(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_home_replaces_leading_tilde() {
        let home = dirs::home_dir().expect("home directory required for this test");
        assert_eq!(expand_home(Path::new("~")), home);
        assert_eq!(
            expand_home(Path::new("~/foo/bar")),
            home.join("foo").join("bar")
        );
    }

    #[test]
    fn expand_home_replaces_home_token() {
        let home = dirs::home_dir().expect("home directory required for this test");
        assert_eq!(expand_home(Path::new("$HOME/foo")), home.join("foo"));
    }

    #[test]
    fn expand_home_leaves_other_paths_unchanged() {
        assert_eq!(
            expand_home(Path::new("relative/path")),
            PathBuf::from("relative/path")
        );
        assert_eq!(expand_home(Path::new("/etc")), PathBuf::from("/etc"));
    }
}

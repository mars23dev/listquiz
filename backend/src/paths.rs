//! Runtime path resolution for the source tree and the packaged release.
//!
//! In development the binary runs from `backend/target/...`, so the data file and
//! the built frontend live next to `backend/Cargo.toml`. A packaged release runs
//! from the folder the user extracted, so both live next to the executable.
//!
//! `LISTQUIZ_ROOT`, `LISTQUIZ_DB`, and `LISTQUIZ_DIST` override the defaults for
//! unusual layouts and manual testing.

use std::path::{Path, PathBuf};

/// The extracted release folder, when the executable sits next to `frontend/dist`.
pub fn package_root() -> Option<PathBuf> {
    if let Some(root) = std::env::var_os("LISTQUIZ_ROOT") {
        return Some(PathBuf::from(root));
    }

    let dir = current_exe_dir()?;
    let index = dir.join("frontend").join("dist").join("index.html");
    index.is_file().then_some(dir)
}

fn current_exe_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()?
        .parent()
        .map(Path::to_path_buf)
}

/// The built frontend the backend serves.
pub fn default_dist_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("LISTQUIZ_DIST") {
        return PathBuf::from(path);
    }
    if let Some(root) = package_root() {
        return root.join("frontend").join("dist");
    }
    let source_dist = source_root().join("frontend").join("dist");
    if source_dist.is_dir() {
        return source_dist;
    }
    current_exe_dir()
        .map(|dir| dir.join("frontend").join("dist"))
        .unwrap_or(source_dist)
}

/// The SQLite file. In development it stays under `backend/data`; in a release
/// zip it is created inside the extracted folder, which keeps all state portable.
pub fn default_db_path() -> PathBuf {
    if let Some(path) = std::env::var_os("LISTQUIZ_DB") {
        return PathBuf::from(path);
    }
    if let Some(root) = package_root() {
        return root.join("data").join("listquiz.db");
    }
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    if manifest.join("Cargo.toml").is_file() {
        return manifest.join("data").join("listquiz.db");
    }
    current_exe_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("data")
        .join("listquiz.db")
}

fn source_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

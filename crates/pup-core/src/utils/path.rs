use std::path::PathBuf;
use std::path::Path;
use std::fs;

/// Treat platforms uniformly regardless of mix and matching formats
pub fn join<U: AsRef<Path>, V: AsRef<Path>>(a: U, b: V) -> PathBuf {
    let mut buffer = PathBuf::new();
    for component in a.as_ref().components() {
        buffer.push(component);
    }
    for component in b.as_ref().components() {
        buffer.push(component);
    }
    return buffer;
}

/// Check if a file exists
pub fn exists<P: AsRef<Path>>(path: P) -> bool {
    return fs::metadata(path.as_ref()).is_ok();
}
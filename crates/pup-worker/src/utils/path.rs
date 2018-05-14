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

/// Return the canonical *display* path for a path.
pub fn display<P: AsRef<Path>>(path: P) -> String {
    adjust_canonicalization(path)
}

#[cfg(not(target_os = "windows"))]
fn adjust_canonicalization<P: AsRef<Path>>(p: P) -> String {
    p.as_ref().display().to_string()
}

#[cfg(target_os = "windows")]
fn adjust_canonicalization<P: AsRef<Path>>(p: P) -> String {
    const VERBATIM_PREFIX: &str = r#"\\?\"#;
    let p = p.as_ref().display().to_string();
    if p.starts_with(VERBATIM_PREFIX) {
        p[VERBATIM_PREFIX.len()..].to_string()
    } else {
        p
    }
}
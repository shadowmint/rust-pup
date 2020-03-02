use base_logging::{Level, Logger};
use pup_worker::errors::{PupWorkerError, PupWorkerErrorType};
use pup_worker::logger;
use std::fmt::Display;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Check if a file exists
pub fn exists<P: AsRef<Path>>(path: P) -> bool {
    fs::metadata(path.as_ref()).is_ok()
}

/// Move a path to another path
pub fn mv<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<(), PupWorkerError> {
    let to_resolved = resolve_to_path(&from, &to)?;
    PupWorkerError::wrap(fs::rename(from, to_resolved))?;
    Ok(())
}

/// Create a path
pub fn mkdir<P: AsRef<Path>>(path: P) -> Result<(), io::Error> {
    fs::create_dir_all(path)
}

/// Archive a path, by moving it some related but unused name
pub fn archive<P: AsRef<Path> + Display>(path: P) -> Result<(), PupWorkerError> {
    if !exists(&path) {
        return Ok(());
    }
    let name = format!("{}--{}", path, Uuid::new_v4());
    mv(path, name)?;
    Ok(())
}

/// Recursive copy a path, or copy a single file
pub fn copy<U: AsRef<Path>, V: AsRef<Path>>(from: U, to: V) -> Result<(), PupWorkerError> {
    let mut logger = logger::get_logger()?;
    if from.as_ref().is_dir() {
        logger.log(Level::Info, format!("copy folder: {:?}", from.as_ref()));
        PupWorkerError::wrap(copy_folder(from, to, &mut logger))?;
    } else {
        logger.log(Level::Info, format!("copy file: {:?}", from.as_ref()));
        PupWorkerError::wrap(copy_file(from, to))?;
    }
    Ok(())
}

pub fn copy_file<U: AsRef<Path>, V: AsRef<Path>>(from: U, to: V) -> Result<(), PupWorkerError> {
    let to_resolved = resolve_to_path(&from, &to)?;
    PupWorkerError::wrap(fs::copy(&from.as_ref(), to_resolved))?;
    Ok(())
}

pub fn copy_folder<U: AsRef<Path>, V: AsRef<Path>>(
    from: U,
    to: V,
    logger: &mut Logger,
) -> Result<(), std::io::Error> {
    let mut stack = Vec::new();
    stack.push(PathBuf::from(from.as_ref()));

    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();

    while let Some(working_path) = stack.pop() {
        logger.log(Level::Info, format!("process: {:?}", &working_path));

        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).is_err() {
            logger.log(Level::Info, format!("  mkdir: {:?}", dest));
            fs::create_dir_all(&dest)?;
        }

        for entry in fs::read_dir(working_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                match path.file_name() {
                    Some(filename) => {
                        let dest_path = dest.join(filename);
                        logger.log(
                            Level::Info,
                            format!("   copy: {:?} -> {:?}", &path, &dest_path),
                        );
                        fs::copy(&path, &dest_path)?;
                    }
                    None => {
                        logger.log(Level::Info, format!("  failed: {:?}", path));
                    }
                }
            }
        }
    }

    Ok(())
}

/// If the destination is a path, try to use the source filename to guess an output name
fn resolve_to_path<P: AsRef<Path>, Q: AsRef<Path>>(
    from: P,
    to: Q,
) -> Result<PathBuf, PupWorkerError> {
    let path = if to.as_ref().is_dir() {
        match from.as_ref().file_name() {
            Some(filename) => to.as_ref().join(filename),
            None => {
                return Err(PupWorkerError::with_message(
                    PupWorkerErrorType::InvalidRequest,
                    &format!("No filename: {:?}", to.as_ref()),
                ));
            }
        }
    } else {
        PathBuf::from(to.as_ref())
    };
    Ok(path)
}

use std::path::Path;
use std::fs::File;
use pup_worker::errors::{PupWorkerError, PupWorkerErrorType};
use serde_yaml;
use std::io::Read;
use std::error::Error;
use pup_worker::logger::get_logger;
use pup_worker::logger::Level;
use pup_worker::utils::path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub message: String
}

impl Manifest {
    pub fn try_from<P: AsRef<Path>>(manifest_path: P) -> Result<Self, PupWorkerError> {
        let mut logger = get_logger()?;
        logger.log(Level::Debug, format!("Trying manifest path: {}", path::display(&manifest_path)));
        return match Self::read_manifest(&manifest_path) {
            Ok(manifest) => Ok(manifest),
            Err(err) => {
                logger.log(Level::Error, format!("Unable to open file: {:?}: {}. Try -h for options.", path::display(&manifest_path), err.description()));
                Err(PupWorkerError::with_message(PupWorkerErrorType::InnerError, "Failed to execute worker"))
            }
        };
    }

    fn read_manifest<P: AsRef<Path>>(manifest_path: P) -> Result<Self, PupWorkerError> {
        if !Path::exists(manifest_path.as_ref()) {
            return Err(PupWorkerError::with_message(PupWorkerErrorType::IOError, "No such file"));
        }
        let mut fp = PupWorkerError::wrap(File::open(&manifest_path))?;
        let mut raw = String::new();
        PupWorkerError::wrap(fp.read_to_string(&mut raw))?;
        return PupWorkerError::wrap(serde_yaml::from_str(&raw));
    }
}
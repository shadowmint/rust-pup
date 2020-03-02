use pup_worker::errors::{PupWorkerError, PupWorkerErrorType};
use pup_worker::logger::get_logger;
use pup_worker::logger::Level;
use pup_worker::utils::path;
use serde_yaml;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub enum ManifestAction {
    Archive,
    Create,
    Require,
    Move,
    Copy,
}

#[derive(Debug, Deserialize)]
pub struct ManifestStep {
    pub path: String,

    pub action: ManifestAction,

    // Optional param to set the destination output
    #[serde(default = "ManifestStep::default_destination")]
    pub destination: String,
}

#[derive(Debug, Deserialize)]
pub struct ManifestConfig {
    pub working_directory: String,
}

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub steps: Vec<ManifestStep>,
    pub config: ManifestConfig,
}

impl ManifestStep {
    fn default_destination() -> String {
        "".to_string()
    }
}

impl Manifest {
    pub fn try_from<P: AsRef<Path>>(manifest_path: P) -> Result<Self, PupWorkerError> {
        let mut logger = get_logger()?;
        logger.log(
            Level::Debug,
            format!("Trying manifest path: {}", path::display(&manifest_path)),
        );
        match Self::read_manifest(&manifest_path) {
            Ok(manifest) => Ok(manifest),
            Err(err) => {
                logger.log(
                    Level::Error,
                    format!(
                        "Unable to open file: {:?}: {}. Try -h for options.",
                        path::display(&manifest_path),
                        err.description()
                    ),
                );
                Err(PupWorkerError::with_message(
                    PupWorkerErrorType::InnerError,
                    "Failed to execute worker",
                ))
            }
        }
    }

    fn read_manifest<P: AsRef<Path>>(manifest_path: P) -> Result<Self, PupWorkerError> {
        if !Path::exists(manifest_path.as_ref()) {
            return Err(PupWorkerError::with_message(
                PupWorkerErrorType::IOError,
                "No such file",
            ));
        }
        let mut fp = PupWorkerError::wrap(File::open(&manifest_path))?;
        let mut raw = String::new();
        PupWorkerError::wrap(fp.read_to_string(&mut raw))?;

        let mut manifest = PupWorkerError::wrap(serde_yaml::from_str(&raw))?;
        Manifest::read_config_from_env(&mut manifest)?;

        Ok(manifest)
    }

    /// Attempt to patch the config settings from the ENV if they
    /// are set. Failure to set a config value is an error.
    fn read_config_from_env(manifest: &mut Manifest) -> Result<(), PupWorkerError> {
        match env::var(manifest.config.working_directory.clone()) {
            Ok(v) => {
                manifest.config.working_directory = v;
            }
            Err(_) => {
                return Err(PupWorkerError::with_message(
                    PupWorkerErrorType::InvalidRequest,
                    &format!(
                        "Missing ENV variable: {}",
                        manifest.config.working_directory
                    ),
                ));
            }
        }
        Ok(())
    }
}

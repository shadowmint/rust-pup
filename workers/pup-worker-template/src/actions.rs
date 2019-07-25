use pup_worker::errors::PupWorkerError;
use ::base_logging::Logger;
use crate::manifest::Manifest;
use pup_worker::logger::Level;

pub fn process_manifest(manifest: Manifest, logger: &mut Logger) -> Result<(), PupWorkerError> {
    logger.log(Level::Info, format!("message: {}", manifest.message));
    return Ok(());
}
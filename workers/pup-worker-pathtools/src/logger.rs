use base_logging::Level;
use base_logging::Logger;
use pup_worker::errors::PupWorkerError;
use pup_worker::logger;

pub fn configure_logging(level: Level) -> Result<(), PupWorkerError> {
    logger::configure_console_logging("pup-worker-pathtools", level)?;
    Ok(())
}

pub fn get_logger() -> Result<Logger, PupWorkerError> {
    logger::get_logger()
}

use pup_worker::logger;
use pup_worker::errors::PupWorkerError;
use base_logging::Logger;
use base_logging::Level;

pub fn configure_logging(level: Level) -> Result<(), PupWorkerError> {
    logger::configure_console_logging("pup-worker-patch", level)?;
    Ok(())
}

pub fn get_logger() -> Result<Logger, PupWorkerError> {
    return logger::get_logger();
}
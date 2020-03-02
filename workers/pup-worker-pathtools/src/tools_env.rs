use pup_worker::errors::{PupWorkerError, PupWorkerErrorType};
use std::env;

/// Get an env variable or return an error.
pub fn get_env(name: &str) -> Result<String, PupWorkerError> {
    match env::var(name) {
        Ok(v) => Ok(v.to_string()),
        Err(_) => Err(PupWorkerError::with_message(
            PupWorkerErrorType::InvalidRequest,
            &format!("Missing ENV variable: {}", name),
        )),
    }
}

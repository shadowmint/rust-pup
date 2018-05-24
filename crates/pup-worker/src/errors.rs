use std::error::Error;
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PupWorkerErrorType {
    InnerError,
    InvalidRequest,
    IOError,
    FailedToSpawnWorker,
    SkipTask,
}

#[derive(Debug)]
pub struct PupWorkerError {
    /// The code that identifies what sort of error this is?
    pub error_type: PupWorkerErrorType,

    /// A human readable error message
    pub error_detail: String,

    /// The inner error detail if any
    pub error_inner: Option<String>,
}

impl PupWorkerError {
    pub fn with_message(error_type: PupWorkerErrorType, error_detail: &str) -> Self {
        return PupWorkerError {
            error_type,
            error_detail: format!("Error: {:?}: {}", error_type, error_detail),
            error_inner: None,
        };
    }

    pub fn with_error<E: Error>(error_type: PupWorkerErrorType, error_detail: &str, inner_error: E) -> Self {
        return PupWorkerError {
            error_type,
            error_detail: format!("Error: {:?}: {}", error_type, error_detail),
            error_inner: Some(inner_error.description().to_string()),
        };
    }

    pub fn wrap<U, V>(result: Result<U, V>) -> Result<U, PupWorkerError> where V: Error {
        match result {
            Ok(u) => Ok(u),
            Err(v) => Err(PupWorkerError::with_error(PupWorkerErrorType::InnerError, "Uncaught error", v))
        }
    }
}

impl From<PupWorkerErrorType> for PupWorkerError {
    fn from(error_type: PupWorkerErrorType) -> Self {
        return PupWorkerError {
            error_type,
            error_detail: format!("Error: {:?}", error_type),
            error_inner: None,
        };
    }
}

impl Error for PupWorkerError {
    fn description(&self) -> &str {
        return &self.error_detail;
    }
}

impl fmt::Display for PupWorkerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}
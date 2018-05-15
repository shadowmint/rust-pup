use std::error::Error;
use std::fmt;
use std::io;
use ::serde_yaml;

#[derive(Debug, Copy, Clone)]
pub enum PupErrorType {
    InnerError,
    MissingVersionFolder,
    MissingVersion,
    MissingWorker,
    MissingManifest,
    MissingProcessManifest,
    RunnerAlreadyCompleted,
    WorkerFailed,
    InvalidRequest,
    MissingArgument,
    FailedToSpawnWorker,
}

#[derive(Debug)]
pub struct PupError {
    /// The code that identifies what sort of error this is?
    pub error_type: PupErrorType,

    /// A human readable error message
    pub error_detail: String,

    /// The inner error if any
    pub error_inner: Option<Box<Error + Send + 'static>>,
}

impl PupError {
    pub fn with_message(error_type: PupErrorType, error_detail: &str) -> Self {
        return PupError {
            error_type,
            error_detail: format!("{:?}: {}", error_type, error_detail),
            error_inner: None,
        };
    }

    pub fn with_error<E: Error + Send + 'static>(error_type: PupErrorType, error_detail: &str, inner_error: E) -> Self {
        return PupError {
            error_type,
            error_detail: format!("Error: {:?}: {}", error_type, error_detail),
            error_inner: Some(Box::new(inner_error) as Box<Error + Send + 'static>),
        };
    }
}

impl From<PupErrorType> for PupError {
    fn from(error_type: PupErrorType) -> Self {
        return PupError {
            error_type,
            error_detail: format!("Error: {:?}", error_type),
            error_inner: None,
        };
    }
}

impl From<io::Error> for PupError {
    fn from(err: io::Error) -> Self {
        return PupError::from(Box::new(err) as Box<Error + Send + 'static>);
    }
}

impl From<serde_yaml::Error> for PupError {
    fn from(err: serde_yaml::Error) -> Self {
        return PupError::from(Box::new(err) as Box<Error + Send + 'static>);
    }
}

impl From<Box<Error + Send + 'static>> for PupError {
    fn from(err: Box<Error + Send + 'static>) -> PupError {
        return PupError {
            error_type: PupErrorType::InnerError,
            error_detail: String::from(err.description()),
            error_inner: Some(err),
        };
    }
}

impl Error for PupError {
    fn description(&self) -> &str {
        return &self.error_detail;
    }
}

impl fmt::Display for PupError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}
use handlebars::TemplateRenderError;
use pup_worker::errors::PupWorkerError;
use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum WorkerError {
    WrappedError(String),
    FailureReturnCode,
}

impl Error for WorkerError {}

impl fmt::Display for WorkerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<TemplateRenderError> for WorkerError {
    fn from(e: TemplateRenderError) -> Self {
        WorkerError::WrappedError(e.description().to_string())
    }
}

impl From<io::Error> for WorkerError {
    fn from(e: io::Error) -> Self {
        WorkerError::WrappedError(e.description().to_string())
    }
}

impl From<PupWorkerError> for WorkerError {
    fn from(e: PupWorkerError) -> Self {
        WorkerError::WrappedError(e.description().to_string())
    }
}

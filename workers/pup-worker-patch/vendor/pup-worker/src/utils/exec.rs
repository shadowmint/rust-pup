use std::process::{Command, Stdio};
use ::errors::{PupWorkerError, PupWorkerErrorType};
use std::path::PathBuf;
use std::collections::HashMap;
use std::error::Error;
use utils::path;

pub struct ExecRequest {
    /// The set of environment variables to add
    pub env: HashMap<String, String>,

    /// Path to the binary to execute
    pub binary_path: PathBuf,

    /// The set of arguments to use
    pub args: Vec<String>,

    /// Should we capture output?
    pub capture: bool,
}

#[derive(Debug)]
pub struct ExecResult {
    /// Return code from executing the binary
    pub return_code: i32,

    /// Output, if we captured it
    pub stdout: Option<String>,
}

pub fn exec(request: ExecRequest) -> Result<ExecResult, PupWorkerError> {
    match request.capture {
        true => exec_capture(request),
        false => exec_stream(request)
    }
}

fn exec_stream(request: ExecRequest) -> Result<ExecResult, PupWorkerError> {
    match Command::new(&request.binary_path)
        .args(&request.args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .envs(&request.env)
        .spawn() {
        Ok(mut cmd) => {
            let return_code = PupWorkerError::wrap(cmd.wait())?;
            return Ok(ExecResult {
                return_code: return_code.code().unwrap(),
                stdout: None,
            });
        }
        Err(err) => {
            return Err(PupWorkerError::with_error(
                PupWorkerErrorType::FailedToSpawnWorker,
                &format!("Unable to spawn worker: {}: {}", path::display(&request.binary_path), err.description()),
                err));
        }
    };
}

fn exec_capture(request: ExecRequest) -> Result<ExecResult, PupWorkerError> {
    match Command::new(&request.binary_path)
        .args(&request.args)
        .envs(&request.env)
        .output() { 
        Ok(output) => {
            let out = String::from_utf8_lossy(&output.stdout);
            let return_code = output.status;
            return Ok(ExecResult {
                return_code: return_code.code().unwrap(),
                stdout: Some(out.to_string()),
            });
        }
        Err(err) => {
            return Err(PupWorkerError::with_error(
                PupWorkerErrorType::FailedToSpawnWorker,
                &format!("Unable to spawn worker: {}: {}", path::display(&request.binary_path), err.description()),
                err));
        }
    };
}
use crate::errors::{PupError, PupErrorType};
use crate::utils::path;
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub struct ExecRequest {
    /// The set of environment variables to add
    pub env: HashMap<String, String>,

    /// Path to the binary to execute
    pub binary_path: PathBuf,

    /// The set of arguments to use
    pub args: Vec<String>,
}

#[derive(Debug)]
pub struct ExecResult {
    /// Return code from executing the binary
    pub return_code: i32,
}

pub fn exec(request: ExecRequest) -> Result<ExecResult, PupError> {
    match Command::new(&request.binary_path)
        .args(&request.args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .envs(&request.env)
        .spawn()
    {
        Ok(mut cmd) => {
            let return_code = cmd.wait()?;
            return Ok(ExecResult {
                return_code: return_code.code().unwrap(),
            });
        }
        Err(err) => {
            return Err(PupError::with_error(
                PupErrorType::FailedToSpawnWorker,
                &format!(
                    "Unable to spawn worker: {}: {}",
                    path::display(&request.binary_path),
                    err.description()
                ),
                err,
            ));
        }
    };
}

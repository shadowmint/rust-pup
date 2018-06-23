use pup_worker::utils::exec::ExecRequest;
use pup_worker::errors::PupWorkerError;
use std::process::Command;
use std::process::Stdio;
use pup_worker::utils::exec::ExecResult;
use pup_worker::errors::PupWorkerErrorType;
use pup_worker::utils::path;
use std::error::Error;

pub fn exec_detached(request: ExecRequest) -> Result<ExecResult, PupWorkerError> {
    match Command::new(&request.binary_path)
        .args(&request.args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .envs(&request.env)
        .spawn() {
        Ok(_) => {
            // Don't wait, just return immediately and fake a 0 return code. 
            // We don't care if the process fails.
            return Ok(ExecResult {
                return_code: 0,
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
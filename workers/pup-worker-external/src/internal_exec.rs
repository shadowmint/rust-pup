use pup_worker::utils::exec::ExecRequest;
use pup_worker::errors::PupWorkerError;
use std::process::Command;
use std::process::Stdio;
use pup_worker::utils::exec::ExecResult;
use pup_worker::errors::PupWorkerErrorType;
use pup_worker::utils::path;
use std::error::Error;
use std::io::{BufReader, BufRead};
use std::thread;
use std::sync::Arc;
use std::sync::Mutex;

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

pub fn exec_stream(request: ExecRequest, on_stdout: impl Fn(&str) + Send + 'static, on_stderr: impl Fn(&str) + Send + 'static) -> Result<ExecResult, PupWorkerError> {
    match Command::new(&request.binary_path)
        .args(&request.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn() {
        Ok(cmd) => {
            let mut cmd_arc = Arc::new(Mutex::new(cmd));

            let mut cmd_stdout_arc = cmd_arc.clone();

            let stdout_handle = thread::spawn(move || {
                let mut stdout_guard = cmd_stdout_arc.lock().unwrap();
                let stdout = stdout_guard.stdout.as_mut().unwrap();
                let stdout_reader = BufReader::new(stdout);
                let stdout_lines = stdout_reader.lines();

                for line in stdout_lines {
                    match line {
                        Ok(lstr) => on_stdout(&lstr),
                        Err(_) => {}
                    }
                }
            });

            let mut cmd_stderr_arc = cmd_arc.clone();
            let stderr_handle = thread::spawn(move || {
                let mut stderr_guard = cmd_stderr_arc.lock().unwrap();
                let stderr = stderr_guard.stderr.as_mut().unwrap();
                let stderr_reader = BufReader::new(stderr);
                let stderr_lines = stderr_reader.lines();

                for line in stderr_lines {
                    match line {
                        Ok(lstr) => on_stderr(&lstr),
                        Err(_) => {}
                    }
                }
            });

            stderr_handle.join().unwrap();
            stdout_handle.join().unwrap();

            let return_code = PupWorkerError::wrap(cmd_arc.lock().unwrap().wait())?;

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
    }
}
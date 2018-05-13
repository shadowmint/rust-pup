use std::process::{Command, Stdio};
use ::errors::PupError;
use std::path::PathBuf;
use std::collections::HashMap;

pub struct ExecRequest {
    /// The set of environment variables to add
    pub env: HashMap<String, String>,

    /// Path to the binary to execute
    pub binary_path: PathBuf,

    /// The set of arguments to use
    pub args: Vec<String>,
}

pub struct ExecResult {
    /// Return code from executing the binary
    pub return_code: i32,
}

pub fn exec(request: ExecRequest) -> Result<ExecResult, PupError> {
    let mut cmd = Command::new(request.binary_path)
        .args(&request.args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .envs(&request.env)
        .spawn()
        .unwrap();
    let return_code = cmd.wait()?;
    return Ok(ExecResult {
        return_code: return_code.code().unwrap()
    });
}

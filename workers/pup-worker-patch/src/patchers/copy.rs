use crate::patch::Patcher;
use std::path::PathBuf;
use crate::manifest::PatchTask;
use pup_worker::errors::PupWorkerError;
use std::fs;

pub struct CopyPatcher {}

impl CopyPatcher {
    pub fn new() -> CopyPatcher {
        return CopyPatcher {};
    }
}

impl Patcher for CopyPatcher {
    fn patch(&mut self, input: PathBuf, output: PathBuf, _: &PatchTask) -> Result<(), PupWorkerError> {
        PupWorkerError::wrap(fs::copy(input, output))?;
        Ok(())
    }
}

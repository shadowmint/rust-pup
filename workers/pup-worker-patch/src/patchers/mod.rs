mod regex;
mod copy;

use manifest::PatchMode;
use pup_worker::errors::{PupWorkerError, PupWorkerErrorType};
use patch::Patcher;

use self::regex::RegexPatcher;
use self::copy::CopyPatcher;

pub fn get_patcher(mode: PatchMode) -> Result<Box<Patcher>, PupWorkerError> {
    return match mode {
        PatchMode::Copy => Ok(Box::new(CopyPatcher::new())),
        PatchMode::Regex => Ok(Box::new(RegexPatcher::new())),
        _ => Err(PupWorkerError::with_message(
            PupWorkerErrorType::InvalidRequest,
            &format!("Unimplemented patch mode: {:?}", mode)))
    };
}
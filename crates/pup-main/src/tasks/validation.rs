use ::PupArg;
use ::pup_core::{PupError, PupErrorType};
use std::collections::HashMap;
use tasks::PupTaskRunner;

pub fn require_key(args: &HashMap<PupArg, String>, required: PupArg) -> Result<(), PupError> {
    if !args.contains_key(&required) {
        return Err(PupError::with_message(
            PupErrorType::MissingArgument,
            &format!("Missing argument: {:?}", required),
        ));
    }
    return Ok(());
}

pub fn is_ok(target: &impl PupTaskRunner) -> Result<(), PupError> {
    if !target.ready() {
        return Err(PupError::from(PupErrorType::InvalidRequest));
    }
    return Ok(());
}
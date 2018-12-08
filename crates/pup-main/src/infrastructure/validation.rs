use crate::PupArg;
use crate::pup_core::{PupError, PupErrorType};
use std::collections::HashMap;
use crate::infrastructure::runner::PupTaskRunner;

pub fn require_key(args: &HashMap<PupArg, String>, required: PupArg) -> Result<(), PupError> {
    if !args.contains_key(&required) {
        return Err(PupError::with_message(
            PupErrorType::MissingArgument,
            &format!("Missing argument: {:?}", required),
        ));
    }
    return Ok(());
}

pub fn boolean_value(args: &HashMap<PupArg, String>, key: PupArg) -> Result<bool, PupError> {
    if !args.contains_key(&key) {
        return Ok(false);
    }

    let value = args[&key].to_string();
    return Ok(value != "" && value != "false" && value != "0");
}

pub fn is_ok(target: &impl PupTaskRunner) -> Result<(), PupError> {
    if !target.ready() {
        return Err(PupError::from(PupErrorType::InvalidRequest));
    }
    return Ok(());
}
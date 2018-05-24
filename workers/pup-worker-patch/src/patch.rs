use pup_worker::errors::{PupWorkerError, PupWorkerErrorType};
use base_logging::Logger;
use manifest::PatchTask;
use patchers::get_patcher;
use pup_worker::utils::path;
use pup_worker::utils::env::EnvHelper;
use pup_worker::logger::Level;
use std::path::PathBuf;


pub trait Patcher {
    fn patch(&mut self, input: PathBuf, output: PathBuf, task: &PatchTask) -> Result<(), PupWorkerError>;
}

pub fn process_patch_task(task: &mut PatchTask, logger: &mut Logger) -> Result<(), PupWorkerError> {
    let mut patcher = get_patcher(task.mode)?;
    let mut env = EnvHelper::new();
    let keys = env.ambient_state().clone();

    // Render paths
    let output = env.expand_single_value(&task.output, &keys)?;
    let path = env.expand_single_value(&task.input, &keys)?;

    // Skip this task?
    let skip = env.expand_single_value(&task.skip, &keys)?;
    let should_skip = skip.trim().len() > 0 && skip != "0" && skip.to_lowercase() != "false";
    if should_skip {
        logger.log(Level::Info, format!("(mode: {:?}) Skipped patch ({}): {} -> {}", task.mode, skip, &path, &output));
        return Err(PupWorkerError::from(PupWorkerErrorType::SkipTask));
    }

    // Check path
    if !path::exists(&path) {
        return Err(PupWorkerError::with_message(
            PupWorkerErrorType::InvalidRequest,
            &format!("Path does not exist: {}", &path)));
    }

    // Check output
    let output_path = PathBuf::from(output);
    let container = output_path.parent();
    if container.is_none() || !path::exists(container.as_ref().unwrap()) {
        return Err(PupWorkerError::with_message(
            PupWorkerErrorType::InvalidRequest,
            &format!("Target path does not exist: {:?}", container.as_ref().unwrap())));
    }
    path::blat(&output_path)?;

    let full_input_path = PupWorkerError::wrap(PathBuf::from(&path).canonicalize())?;
    let full_output_path = PupWorkerError::wrap(output_path.canonicalize())?;
    if full_input_path == full_output_path {
        return Err(PupWorkerError::with_message(
            PupWorkerErrorType::InvalidRequest,
            &format!("Cannot write and read from file {} at the same time", path::display(full_input_path)),
        ));
    }

    // Render values
    // NB. We don't render patterns, because they will certainly be weird.
    for step in task.patch.iter_mut() {
        for value in step.values.iter_mut() {
            *value = env.expand_single_value(value, &keys)?;
        }
    }

    // Patch path
    logger.log(Level::Info, format!("(mode: {:?}) Patching target: {} -> {}", task.mode, path::display(&path), path::display(&full_output_path)));
    patcher.patch(full_input_path, full_output_path, task)?;

    Ok(())
}
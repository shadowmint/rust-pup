use crate::manifest::{Manifest, ManifestAction, ManifestStep};
use ::base_logging::Logger;
use pup_worker::errors::{PupWorkerError, PupWorkerErrorType};
use pup_worker::logger::{get_logger, Level};
use std::env;

pub fn process_manifest(manifest: Manifest, logger: &mut Logger) -> Result<(), PupWorkerError> {
    set_cwd(&manifest, logger)?;
    for step in manifest.steps.iter() {
        apply_step(step)?;
    }
    Ok(())
}

fn set_cwd(manifest: &Manifest, logger: &mut Logger) -> Result<(), PupWorkerError> {
    logger.log(
        Level::Info,
        format!(
            "Setting working directory from config: {}",
            &manifest.config.working_directory
        ),
    );
    match env::set_current_dir(&manifest.config.working_directory) {
        Ok(_) => {}
        Err(err) => {
            return Err(PupWorkerError::with_message(
                PupWorkerErrorType::IOError,
                &format!("Failed to set active path: {}", err),
            ));
        }
    };
    Ok(())
}

fn apply_step(step: &ManifestStep) -> Result<(), PupWorkerError> {
    let mut logger = get_logger()?;
    let resolved_path = crate::tools_env::get_env(&step.path)?;
    let resolved_destination = match step.destination.as_str() {
        "" => "".to_string(),
        v => crate::tools_env::get_env(&v)?,
    };

    if resolved_destination.is_empty() {
        logger.log(
            Level::Info,
            format!("Apply: {:?}: {}", step.action, &resolved_path),
        );
    } else {
        logger.log(
            Level::Info,
            format!(
                "Apply: {:?}: {} -> {}",
                step.action, &resolved_path, &resolved_destination
            ),
        );
    }

    match step.action {
        ManifestAction::Archive => apply_archive(resolved_path),
        ManifestAction::Create => apply_create(resolved_path),
        ManifestAction::Require => apply_require(resolved_path),
        ManifestAction::Move => apply_move(resolved_path, resolved_destination),
        ManifestAction::Copy => apply_copy(resolved_path, resolved_destination),
    }?;

    Ok(())
}

fn apply_archive(path: String) -> Result<(), PupWorkerError> {
    match crate::tools_path::archive(path) {
        Ok(_) => Ok(()),
        Err(err) => Err(PupWorkerError::with_message(
            PupWorkerErrorType::IOError,
            &format!("{}", err),
        )),
    }
}

fn apply_create(path: String) -> Result<(), PupWorkerError> {
    match crate::tools_path::mkdir(path) {
        Ok(_) => Ok(()),
        Err(err) => Err(PupWorkerError::with_message(
            PupWorkerErrorType::IOError,
            &format!("{}", err),
        )),
    }
}

fn apply_require(path: String) -> Result<(), PupWorkerError> {
    if !crate::tools_path::exists(&path) {
        return apply_create(path);
    }
    Ok(())
}

fn apply_move(from: String, to: String) -> Result<(), PupWorkerError> {
    match crate::tools_path::mv(from, to) {
        Ok(_) => Ok(()),
        Err(err) => Err(PupWorkerError::with_message(
            PupWorkerErrorType::IOError,
            &format!("{}", err),
        )),
    }
}

fn apply_copy(from: String, to: String) -> Result<(), PupWorkerError> {
    match crate::tools_path::copy(from, to) {
        Ok(_) => Ok(()),
        Err(err) => Err(PupWorkerError::with_message(
            PupWorkerErrorType::IOError,
            &format!("{}", err),
        )),
    }
}

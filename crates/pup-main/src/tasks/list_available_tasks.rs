extern crate pup_core;

use ::PupArg;
use ::pup_core::PupError;
use ::infrastructure::validation;
use std::collections::HashMap;
use std::path::PathBuf;
use pup_core::utils::path;
use ::base_logging::{Logger, Level};
use ::pup_core::PupProcess;
use infrastructure::runner::PupTaskRunner;
use std::error::Error;

pub fn list_available_tasks() -> TaskRunnerListAvailable {
    return TaskRunnerListAvailable {
        ok: false,
        path: PathBuf::new(),
        show_versions: false,
    };
}

pub struct TaskRunnerListAvailable {
    ok: bool,
    path: PathBuf,
    show_versions: bool,
}

impl PupTaskRunner for TaskRunnerListAvailable {
    fn prepare(&mut self, args: HashMap<PupArg, String>) -> Result<(), PupError> {
        validation::require_key(&args, PupArg::ProcessManifestPath)?;

        self.path = PathBuf::from(args.get(&PupArg::ProcessManifestPath).unwrap());
        self.ok = true;
        self.show_versions = validation::boolean_value(&args, PupArg::ListTaskVersions)?;

        return Ok(());
    }

    fn ready(&self) -> bool {
        return self.ok;
    }

    fn run(&mut self, logger: &mut Logger) -> Result<(), PupError> {
        validation::is_ok(self)?;

        logger.log(Level::Debug, format!("Reading: {}", path::display(&self.path)));
        let mut process = PupProcess::load_from(&self.path, None)?;

        logger.log(Level::Debug, format!("Found {} tasks", &process.manifest.tasks.len()));
        for task in &process.manifest.tasks.clone() {
            if self.show_versions {
                match process.task(&task) {
                    Ok((task_ref, _)) => {
                        for version in 0..task_ref.manifest.versions.len() {
                            if  version ==    task_ref.manifest.versions.len() -1 {
                                logger.log(Level::Info, format!("{}#{} (default)", task, task_ref.manifest.versions[version].version));
                            }
                            else {
                                logger.log(Level::Info, format!("{}#{}", task, task_ref.manifest.versions[version].version));    
                            }
                            
                        }
                    }
                    Err(err) => {
                        logger.log(Level::Debug, format!("Failed to process task: {}", err.description()));
                        return Err(err);
                    }
                }
            } else {
                logger.log(Level::Info, format!("{}", task));
            }
        }

        Ok(())
    }
}
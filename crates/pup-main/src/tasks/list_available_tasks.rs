extern crate pup_core;

use ::tasks::PupTaskRunner;
use ::PupArg;
use ::pup_core::PupError;
use ::tasks::validation;
use std::collections::HashMap;
use std::path::PathBuf;
use pup_core::logger::get_logger;
use ::base_logging::{Logger, Level};
use ::pup_core::PupProcess;

pub fn list_available_tasks() -> Box<PupTaskRunner> {
    return Box::new(TaskRunnerListAvailable {
        ok: false,
        path: PathBuf::new(),
    });
}

struct TaskRunnerListAvailable {
    ok: bool,
    path: PathBuf,
}

impl PupTaskRunner for TaskRunnerListAvailable {
    fn prepare(&mut self, args: HashMap<PupArg, String>) -> Result<(), PupError> {
        validation::require_key(&args, PupArg::ProcessManifestPath)?;

        self.path = PathBuf::from(args.get(&PupArg::ProcessManifestPath).unwrap());
        self.ok = true;

        return Ok(());
    }

    fn run(&mut self, logger: &mut Logger) -> Result<u32, PupError> {
        validation::is_ok(self);
        
        logger.log(Level::Info, format!("Reading process: {:?}", self.path));
        let process = PupProcess::load_from(&self.path)?;

        logger.log(Level::Info, format!("Found {} tasks", process.manifest.tasks.len()));
        for task in process.manifest.tasks {
            logger.log(Level::Info, format!("- {}", task));
        }
        
        Ok(0)
    }

    fn ready(&self) -> bool {
        return self.ok;
    }
}
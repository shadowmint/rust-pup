extern crate pup_core;

use ::PupArg;
use ::pup_core::PupError;
use ::pup_core::utils::path;
use ::infrastructure::validation;
use std::collections::HashMap;
use std::path::PathBuf;
use ::base_logging::{Logger, Level};
use ::pup_core::PupProcess;
use infrastructure::runner::PupTaskRunner;
use std::error::Error;

pub fn show_execution_plan() -> TaskRunnerShowPlan {
    return TaskRunnerShowPlan {
        ok: false,
        path: PathBuf::new(),
        task: String::new(),
    };
}

pub struct TaskRunnerShowPlan {
    ok: bool,
    path: PathBuf,
    task: String,
}

impl PupTaskRunner for TaskRunnerShowPlan {
    fn prepare(&mut self, args: HashMap<PupArg, String>) -> Result<(), PupError> {
        validation::require_key(&args, PupArg::ProcessManifestPath)?;
        validation::require_key(&args, PupArg::TaskId)?;

        self.path = PathBuf::from(args.get(&PupArg::ProcessManifestPath).unwrap());
        self.task = args.get(&PupArg::TaskId).unwrap().to_string();
        self.ok = true;

        return Ok(());
    }

    fn ready(&self) -> bool {
        return self.ok;
    }

    fn run(&mut self, logger: &mut Logger) -> Result<(), PupError> {
        validation::is_ok(self)?;

        logger.log(Level::Debug, format!("Reading: {:?}", path::display(&self.path)));
        let mut process = PupProcess::load_from(&self.path, None)?;

        logger.log(Level::Debug, format!("Opening: {}", self.task));
        match process.runner(&self.task) {
            Ok(runner) => {
                logger.log(Level::Info, format!("{:?}", runner));
                Ok(())
            }
            Err(err) => {
                logger.log(Level::Debug, format!("Failed: {}", err.description()));
                Err(err)
            }
        }
    }
}
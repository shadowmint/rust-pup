use ::tasks::PupTaskRunner;
use ::PupArg;
use ::pup_core::PupError;
use std::collections::HashMap;

pub fn list_available_tasks() -> Box<PupTaskRunner> {
    return Box::new(TaskRunner {});
}

struct TaskRunner {}

impl PupTaskRunner for TaskRunner {
    fn prepare(&mut self, args: HashMap<PupArg, String>) -> Result<(), PupError> {
        Ok(())
    }

    fn run(&mut self) -> Result<u32, PupError> {
        Ok(0)
    }
}
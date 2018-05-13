extern crate pup_core;

mod tasks;

use pup_core::{PupError, PupErrorType};
use std::collections::HashMap;
use tasks::PupTaskRunner;
use tasks::list_available_tasks::list_available_tasks;

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum PupArg {
    ProcessManifestPath,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PupTask {
    ListAvailableTasks
}

pub fn pup_main(task: PupTask, args: HashMap<PupArg, String>) -> Result<u32, PupError> {
    return match get_runner(task) {
        Some(mut runner) => {
            let is_valid = runner.prepare(args);
            if is_valid.is_err() {
                return Err(is_valid.err().unwrap());
            }
            return runner.run();
        }
        None => Err(PupError::with_message(
            PupErrorType::InvalidRequest,
            &format!("Unsupported action: {:?}", task),
        ))
    };
}

fn get_runner(task: PupTask) -> Option<Box<PupTaskRunner>> {
    if task == PupTask::ListAvailableTasks {
        return Some(list_available_tasks());
    }
    return None;
}

#[cfg(test)]
mod tests {
    use super::{pup_main, PupArg, PupTask};
    use super::pup_core::testing::test_context_process_path;
    use std::collections::HashMap;

    #[test]
    fn run_sample()
    {
        let mut args = HashMap::new();
        args.insert(PupArg::ProcessManifestPath, String::from(test_context_process_path().to_str().unwrap()));

        let result = pup_main(PupTask::ListAvailableTasks, args);
        assert!(result.is_ok());
    }
}

extern crate base_logging;
extern crate pup_core;

mod tasks;
mod infrastructure;

use pup_core::{PupError, PupErrorType};
use std::collections::HashMap;
use crate::tasks::get_task_runner;
use pup_core::logger::set_logger_level;
use base_logging::Level;

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum PupArg {
    ProcessManifestPath,
    ListTaskVersions,
    TaskId,
    DryRun,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PupTask {
    ListAvailableTasks,
    ShowExecutionPlan,
    RunTask,
}

pub fn pup_main(task: PupTask, args: HashMap<PupArg, String>) -> Result<(), PupError> {
    return match get_task_runner(task) {
        Some(mut runner) => {
            let is_valid = runner.prepare(args);
            if is_valid.is_err() {
                return Err(is_valid.err().unwrap());
            }
            let mut logger = ::pup_core::logger::get_logger();
            logger.log(Level::Debug, format!("Executing task: {:?}", task));
            return runner.run(&mut logger);
        }
        None => Err(PupError::with_message(
            PupErrorType::InvalidRequest,
            &format!("Unsupported action: {:?}", task),
        ))
    };
}

/// Enable verbose debugging.
pub fn pup_enable_debug() {
    set_logger_level(Level::Debug);
}

#[cfg(test)]
mod tests {
    use super::{pup_main, PupArg, PupTask};
    use super::pup_core::testing::test_context_process_path;
    use std::collections::HashMap;
    use pup_core::logger::set_logger_level;
    use base_logging::Level;

    #[test]
    fn test_show_tasks()
    {
        set_logger_level(Level::Debug);
        let mut args = HashMap::new();
        args.insert(PupArg::ProcessManifestPath, String::from(test_context_process_path().to_str().unwrap()));
        args.insert(PupArg::ListTaskVersions, String::from("1"));

        let result = pup_main(PupTask::ListAvailableTasks, args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_tasks_simple()
    {
        set_logger_level(Level::Debug);
        let mut args = HashMap::new();
        args.insert(PupArg::ProcessManifestPath, String::from(test_context_process_path().to_str().unwrap()));

        let result = pup_main(PupTask::ListAvailableTasks, args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_plan()
    {
        set_logger_level(Level::Debug);
        let mut args = HashMap::new();
        args.insert(PupArg::ProcessManifestPath, String::from(test_context_process_path().to_str().unwrap()));
        args.insert(PupArg::TaskId, String::from("tests.builds.deployment"));

        let result = pup_main(PupTask::ShowExecutionPlan, args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_task_dry_run()
    {
        set_logger_level(Level::Debug);
        let mut args = HashMap::new();
        args.insert(PupArg::ProcessManifestPath, String::from(test_context_process_path().to_str().unwrap()));
        args.insert(PupArg::TaskId, String::from("tests.builds.deployment"));
        args.insert(PupArg::DryRun, String::from("1"));

        let result = pup_main(PupTask::RunTask, args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_task()
    {
        set_logger_level(Level::Debug);
        let mut args = HashMap::new();
        args.insert(PupArg::ProcessManifestPath, String::from(test_context_process_path().to_str().unwrap()));
        args.insert(PupArg::TaskId, String::from("tests.builds.deployment"));

        // This will fail because the demo tasks are invalid.
        let _ = pup_main(PupTask::RunTask, args);
    }
}

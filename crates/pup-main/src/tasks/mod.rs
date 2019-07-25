mod list_available_tasks;
mod show_execution_plan;
mod run_task;

use crate::PupTask;
use crate::tasks::list_available_tasks::list_available_tasks;
use crate::infrastructure::runner::PupTaskRunner;
use crate::tasks::show_execution_plan::show_execution_plan;
use crate::tasks::run_task::run_task;

pub fn get_task_runner(task: PupTask) -> Option<Box<PupTaskRunner>> {
    if task == PupTask::ListAvailableTasks {
        return Some(Box::new(list_available_tasks()));
    }
    if task == PupTask::ShowExecutionPlan {
        return Some(Box::new(show_execution_plan()));
    }
    if task == PupTask::RunTask {
        return Some(Box::new(run_task()));
    }
    return None;
}
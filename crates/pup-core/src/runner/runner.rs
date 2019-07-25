use crate::context::PupContext;
use crate::errors::{PupError, PupErrorType};
use crate::logger::get_logger;
use crate::runner::action::PupActionOptions;
use crate::runner::PupAction;
use crate::utils::path;
use std::fmt;

/// A set of tasks to be run
#[derive(Clone)]
pub struct PupRunner {
    /// The context for everything
    context: PupContext,

    /// The root action to run
    root: PupAction,
}

impl PupRunner {
    /// Create a new runner with a context
    pub fn new(context: &PupContext) -> PupRunner {
        return PupRunner {
            context: context.clone(),
            root: PupAction::new(),
        };
    }

    /// Add the entire DAG for a runner from a base task.
    /// name should be a standard format name, eg. foo.bar.foobar#1.0.0
    /// or, to just use whatever the latest version is, foo.bar.foobar
    pub fn add(&mut self, name: &str) -> Result<(), PupError> {
        let mut action = PupAction::new();
        action.load(
            &self.context,
            name,
            &self.context.global_env,
            &self.context.env,
        )?;
        self.root.children.push(action);
        return Ok(());
    }

    /// Actually go and execute all the actions.
    /// The args should be any extra arguments to invoke on all workers, eg. config file.
    pub fn run(&mut self, options: PupActionOptions) -> Result<(), PupError> {
        if self.root.completed {
            return Err(PupError::from(PupErrorType::RunnerAlreadyCompleted));
        }

        let mut logger = get_logger();
        for child in self.root.children.iter_mut() {
            child.run(&mut logger, &options)?;
        }

        return Ok(());
    }

    /// Return a copy of the internal action; for cloning, testing, etc.
    pub fn tasks(&self) -> PupAction {
        return self.root.clone();
    }
}

impl fmt::Debug for PupRunner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let count = self.root.children.len();
        for i in 0..count {
            let child = &self.root.children[i];
            debug_print(f, child, 1, i == count - 1);
        }
        write!(f, "")
    }
}

fn debug_print(f: &mut fmt::Formatter, action: &PupAction, offset: usize, is_last: bool) {
    let ext = action.external.as_ref().unwrap();

    // Name
    let _ = write!(f, " ");
    let _ = write!(f, "{}", "-".repeat(offset));
    let _ = write!(f, " {} #{}", ext.task.name, ext.version.version);

    // Action
    let _ = write!(
        f,
        " ({} -> {})",
        ext.worker.name,
        path::display(&ext.version.path)
    );
    if !is_last || action.children.len() > 0 {
        let _ = write!(f, "\n");
    }

    let count = action.children.len();
    for i in 0..count {
        let child = &action.children[i];
        debug_print(f, child, offset + 1, is_last && i == count - 1);
    }
}

#[cfg(test)]
mod tests {
    use super::PupRunner;
    use crate::runner::action::PupActionOptions;
    use crate::testing::test_fixture;

    #[test]
    fn load_runner_from_working_task() {
        let process = test_fixture();
        let mut runner = PupRunner::new(&process.context);
        assert!(runner.add("tests.builds.deployment").is_ok());
        println!("{:?}", runner);
    }

    #[test]
    fn load_runner_from_invalid_task() {
        let process = test_fixture();
        let mut runner = PupRunner::new(&process.context);
        assert!(runner.add("tests.builds.bad").is_err());
    }

    #[test]
    fn run_runner_in_dry_run_mode() {
        let process = test_fixture();
        let mut runner = PupRunner::new(&process.context);
        assert!(runner.add("tests.builds.deployment").is_ok());
        assert!(runner
            .run(PupActionOptions {
                dry_run: true,
                args: vec!("Hello", "--world", "1")
                    .iter()
                    .map(|x| String::from(*x))
                    .collect(),
            })
            .is_ok());
    }

    #[test]
    fn run_runner_fails_with_invalid_workers_in_real_mode() {
        let process = test_fixture();
        let mut runner = PupRunner::new(&process.context);
        assert!(runner.add("tests.builds.deployment").is_ok());
        assert!(runner
            .run(PupActionOptions {
                dry_run: false,
                args: vec!("config.json")
                    .iter()
                    .map(|x| String::from(*x))
                    .collect(),
            })
            .is_err());
    }

    #[test]
    fn test_rendered_step_env_values() {
        let process = test_fixture();
        let mut runner = PupRunner::new(&process.context);
        assert!(runner.add("tests.actions.nested").is_ok());

        let parent_task = &mut runner.root.children[0].children[0];
        assert!(parent_task.external.is_some());

        let external = parent_task.external.take().unwrap();
        assert_eq!(external.task.name, "tests.common.prepFolder");
        assert_eq!(external.env["foo"], "bar");
        assert_eq!(external.env["bar"], "foobar");
        assert_eq!(external.env["PREP_FOLDER_PATH"], "bar/foobar/nested");
        assert_eq!(external.env["PREP_FOLDER_RULE"], "clean");
    }
}

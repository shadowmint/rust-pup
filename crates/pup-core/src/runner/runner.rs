use std::fmt;
use ::context::PupContext;
use ::errors::{PupError, PupErrorType};
use ::runner::PupAction;
use utils::path;
use logger::get_logger;
use runner::action::PupActionOptions;

/// A set of tasks to be run
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

    /// Add the entire DAG for a runner from a base task
    /// name should be a standard format name, eg. foo.bar.foobar#1.0.0
    /// or, to just use whatever the latest version is, foo.bar.foobar
    pub fn add(&mut self, name: &str) -> Result<(), PupError> {
        let mut action = PupAction::new();
        action.load(&self.context, name)?;
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
}

impl fmt::Debug for PupRunner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for child in self.root.children.iter() {
            debug_print(f, child, 1);
        }
        write!(f, "")
    }
}

fn debug_print(f: &mut fmt::Formatter, action: &PupAction, offset: usize) {
    let ext = action.external.as_ref().unwrap();

    // Name
    let _ = write!(f, " ");
    let _ = write!(f, "{}", "-".repeat(offset));
    let _ = write!(f, " {} #{}", ext.task.name, ext.version.version);

    // Action
    let _ = write!(f, " ({} -> {})\n", ext.worker.name, path::display(&ext.version.path));

    for child in action.children.iter() {
        debug_print(f, child, offset + 1);
    }
}

#[cfg(test)]
mod tests {
    use ::context::PupContext;
    use super::PupRunner;
    use std::env::current_dir;
    use utils::path::join;
    use std::path::PathBuf;
    use ::testing::test_context_fixture;
    use runner::action::PupActionOptions;

    #[test]
    fn load_runner_from_working_task() {
        let context = test_context_fixture();
        let mut runner = PupRunner::new(&context);
        assert!(runner.add("tests.builds.deployment").is_ok());
        println!("{:?}", runner);
    }

    #[test]
    fn load_runner_from_invalid_task() {
        let context = test_context_fixture();
        let mut runner = PupRunner::new(&context);
        assert!(runner.add("tests.builds.bad").is_err());
    }

    #[test]
    fn run_runner_in_dry_run_mode() {
        let context = test_context_fixture();
        let mut runner = PupRunner::new(&context);
        assert!(runner.add("tests.builds.deployment").is_ok());
        assert!(runner.run(PupActionOptions {
            dry_run: true,
            args: vec!("Hello", "--world", "1").iter().map(|x| String::from(*x)).collect(),
        }).is_ok());
    }

    #[test]
    fn run_runner_fails_with_invalid_workers_in_real_mode() {
        let context = test_context_fixture();
        let mut runner = PupRunner::new(&context);
        assert!(runner.add("tests.builds.deployment").is_ok());
        assert!(runner.run(PupActionOptions {
            dry_run: false,
            args: vec!("config.json").iter().map(|x| String::from(*x)).collect(),
        }).is_err());
    }
}
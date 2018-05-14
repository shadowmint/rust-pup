use ::context::PupContext;
use ::manifest::PupManifest;
use ::errors::{PupError};
use ::utils::path::join;
use std::path::Path;
use std::path::PathBuf;

/// A single folder with an action in it is a task.
pub struct PupTask {
    /// The original name of this task
    pub name: String,

    /// The context folder
    //context: PupContext,

    /// The manifest data
    pub manifest: PupManifest,

    /// The path to the folder for this task
    pub path: PathBuf,
}

impl PupTask {
    /// Create a new Task 
    pub fn new<P: AsRef<Path>>(context: PupContext, name: &str, path: P) -> Result<PupTask, PupError> {
        let manifest_path = join(&context.tasks, path);
        match PupManifest::try_from(&manifest_path) {
            Ok(manifest) => {
                let task = PupTask {
                    name: String::from(name),
                   // context,
                    manifest,
                    path: PathBuf::from(manifest_path),
                };
                return Ok(task);
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PupTask;
    use ::context::PupContext;
    use ::testing::test_context_fixture;
    use std::env::current_dir;
    use std::path::PathBuf;
    use std::path::Path;
    use utils::path::join;

    #[test]
    fn load_simple_task() {
        let context = test_context_fixture();
        let task = PupTask::new(context, "tests.actions.setVersion", "tests/actions/setVersion").unwrap();
        assert_eq!(task.manifest.versions.len(), 2);
    }
}

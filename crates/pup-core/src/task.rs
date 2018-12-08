use crate::context::PupContext;
use crate::manifest::PupManifest;
use crate::errors::PupError;
use crate::utils::path::join;
use std::path::Path;
use std::path::PathBuf;

/// A single folder with an action in it is a task.
#[derive(Clone)]
pub struct PupTask {
    /// The original name of this task
    pub name: String,

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
    use crate::testing::test_fixture;

    #[test]
    fn load_simple_task() {
        let process = test_fixture();/*k*/
        let task = PupTask::new(process.context.clone(),
                                "tests.actions.setVersion",
                                "tests/actions/setVersion").unwrap();
        assert_eq!(task.manifest.versions.len(), 2);
    }
}

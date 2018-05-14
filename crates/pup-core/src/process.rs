use ::errors::{PupError, PupErrorType};
use ::utils::path::join;

use ::serde_yaml;

use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::error::Error;
use context::PupContext;
use std::collections::HashMap;
use task::PupTask;
use manifest::PupManifestVersion;
use runner::PupRunner;

#[derive(Debug)]
pub struct PupProcess {
    pub manifest: PupProcessManifest,

    /// The root path to the process sequence
    pub path: PathBuf,

    /// The internal context instance
    _context: Option<PupContext>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PupProcessManifest {
    /// The set of root level tasks to expose
    pub tasks: Vec<String>,

    /// The folder to use for workers
    pub workers_path: String,

    /// The tasks to use for tasks
    pub tasks_path: String,

    /// The path to the environment to use for tasks
    pub environment: HashMap<String, String>,
}

impl PupProcessManifest {
    pub fn try_from(process_manifest_path: &Path) -> Result<Self, PupError> {
        return Self::read_manifest(process_manifest_path).map_err(|err| {
            return PupError::with_error(
                PupErrorType::MissingProcessManifest,
                &format!("Unable to read process manifest: {:?}: {:?}", process_manifest_path, err.description()),
                err,
            );
        });
    }

    fn read_manifest(process_manifest_path: &Path) -> Result<Self, PupError> {
        let mut fp = File::open(&process_manifest_path)?;
        let mut raw = String::new();
        fp.read_to_string(&mut raw)?;

        let mut manifest: PupProcessManifest = serde_yaml::from_str(&raw)?;
        manifest.validate()?;

        return Ok(manifest);
    }

    /// Check process manifest
    pub fn validate(&mut self) -> Result<(), PupError> {
        Ok(())
    }
}

impl PupProcess {
    pub fn load_from<P: AsRef<Path>>(path: P) -> Result<PupProcess, PupError> {
        let manifest = PupProcessManifest::try_from(path.as_ref())?;
        Ok(PupProcess {
            manifest,
            path: PathBuf::from(path.as_ref()),
            _context: None,
        })
    }

    fn process_path(&self) -> &Path {
        return self.path.parent().unwrap();
    }

    pub fn context(&mut self) -> &PupContext {
        if self._context.is_none() {
            let mut context = PupContext::new(
                &join(&self.process_path(), &self.manifest.tasks_path),
                &join(&self.process_path(), &self.manifest.workers_path));
            context.set_environment(&self.manifest.environment);
            self._context = Some(context);
        }
        return self._context.as_ref().unwrap();
    }

    pub fn task(&mut self, task: &str) -> Result<(PupTask, PupManifestVersion), PupError> {
        return self.context().load_task(task);
    }

    pub fn runner(&mut self, task: &str) -> Result<PupRunner, PupError> {
        let mut runner = PupRunner::new(self.context());
        let _ = runner.add(task)?;
        return Ok(runner);
    }
}

#[cfg(test)]
mod tests {
    use super::PupProcess;
    use ::testing::test_context_process_path;

    #[test]
    fn test_load_from_folder() {
        let sample_process = test_context_process_path();
        let process = PupProcess::load_from(sample_process).unwrap();
        assert_eq!(process.manifest.environment.len(), 2);
        assert_eq!(process.manifest.environment["foo"], "bar");
        let context = process.context();
    }

    #[test]
    fn test_root_level_tasks() {
        let sample_process = test_context_process_path();
        let process = PupProcess::load_from(sample_process).unwrap();
        assert_eq!(process.manifest.tasks.len(), 2);
    }
}
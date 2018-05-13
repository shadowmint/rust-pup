use std::path::{Path, PathBuf};
use ::task::PupTask;
use ::errors::PupError;
use ::worker::PupWorker;
use errors::PupErrorType;
use manifest::PupManifestVersion;
use ::logger::get_logger;
use utils::path::join;
use utils::path::exists;
use std::collections::HashMap;

#[derive(Clone)]
pub struct PupContext {
    /// The config file passed to each worker.
    pub env: Option<HashMap<String, String>>,

    /// The root folder for the set of workers which are available.
    pub workers: PathBuf,

    /// The root folder for the set of tasks which are available.
    pub tasks: PathBuf,
}

impl PupContext {
    /// Create a new context with a reference to the tasks
    /// root folder, the workers root folder and the config file.
    pub fn new(tasks: &Path, workers: &Path) -> PupContext {
        return PupContext {
            env: None,
            tasks: PathBuf::from(tasks),
            workers: PathBuf::from(workers),
        };
    }

    /// Import an entire environment settings map
    pub fn set_environment(&mut self, env: &HashMap<String, String>) {
        self.env = Some(env.clone());
    }

    /// Load a context by 'name string' in the format foo.bar.foobar#version
    pub fn load_task(&self, name: &str) -> Result<(PupTask, PupManifestVersion), PupError> {
        let mut logger = get_logger();

        // Extract version & ident from name
        let mut ident = String::from(name);
        let mut version_ident: Option<String> = None;
        if name.contains("#") {
            let parts = name.split("#").collect::<Vec<&str>>();
            ident = String::from(parts[0]);
            version_ident = Some(String::from(parts[1]));
        }

        // Convert ident into path
        let path = ident.replace(".", "/");

        // Try to load
        let task = PupTask::new(self.clone(), &ident, path)?;

        // Check the required version exists
        let version: PupManifestVersion = match version_ident {
            Some(version_id) => {
                let matched = task.manifest.versions.iter().find(|v| { return v.version == version_id; });
                if matched.is_none() {
                    return Err(PupError::with_message(
                        PupErrorType::MissingVersion,
                        &format!("No version matching '{:?}' on {:?}", version_id, task.path),
                    ));
                }
                matched.unwrap().clone()
            }
            None => {
                if !task.manifest.versions.len() == 0 {
                    return Err(PupError::with_message(
                        PupErrorType::MissingVersion,
                        &format!("No versions available on {:?}", task.path),
                    ));
                }
                task.manifest.versions[task.manifest.versions.len() - 1].clone()
            }
        };

        return Ok((task, version));
    }

    /// Find a worker by the name 'name' in the workers folder, and return a PupWorker for it.
    pub fn load_worker(&self, name: &str) -> Result<PupWorker, PupError> {
        // TODO: Cache results.
        let attemp1 = join(&self.workers, name);
        if exists(&attemp1) {
            return Ok(PupWorker {
                path: attemp1,
                name: String::from(name),
                env: self.env.as_ref().map_or_else(|| HashMap::new(), |v| v.clone()),
            });
        }

        let attemp2 = join(&self.workers, join(name, ".exe"));
        if exists(&attemp2) {
            return Ok(PupWorker {
                path: attemp2,
                name: String::from(name),
                env: self.env.as_ref().map_or_else(|| HashMap::new(), |v| v.clone()),
            });
        }

        return Err(PupError::with_message(
            PupErrorType::MissingWorker,
            &format!("Unable to find any worker '{:?}' in {:?}", name, self.workers),
        ));
    }
}

#[cfg(test)]
mod tests {
    use utils::path::join;
    use std::path::PathBuf;
    use ::testing::test_context_fixture;

    #[test]
    fn load_simple_task() {
        let context = test_context_fixture();
        let (task, version) = context.load_task("tests.actions.setVersion#0.0.2").unwrap();
        assert_eq!(task.manifest.versions.len(), 2);
        assert_eq!(version.version, "0.0.2");
    }

    #[test]
    fn fails_to_load_missing_version() {
        let context = test_context_fixture();
        let task = context.load_task("tests.actions.setVersion#1.0.0");
        assert!(task.is_err());
    }
}

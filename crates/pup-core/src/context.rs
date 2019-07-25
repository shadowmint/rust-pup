use self::global_env::build_global_env;
use crate::errors::PupError;
use crate::errors::PupErrorType;
use crate::logger::get_logger;
use crate::manifest::PupManifestVersion;
use crate::task::PupTask;
use crate::utils::path;
use crate::utils::path::exists;
use crate::utils::path::join;
use crate::worker::PupWorker;
use base_logging::Level;
use std::collections::HashMap;
use std::fs::canonicalize;
use std::path::{Path, PathBuf};

mod global_env;

#[derive(Clone, Debug)]
pub struct PupContext {
    /// The global config that exists as a root for the whole process
    pub global_env: HashMap<String, String>,

    /// The config file passed to each worker.
    pub env: HashMap<String, String>,

    /// The root folder for the set of workers which are available.
    pub workers: PathBuf,

    /// The root folder for the set of tasks which are available.
    pub tasks: PathBuf,
}

impl PupContext {
    /// Create a new context with a reference to the tasks
    /// root folder, the workers root folder and the config file.
    pub fn new(tasks: &Path, workers: &Path, root: &Path) -> Result<PupContext, PupError> {
        return Ok(PupContext {
            env: HashMap::new(),
            global_env: build_global_env(root),
            tasks: canonicalize(PathBuf::from(tasks)).map_err(|_e| {
                PupError::with_message(
                    PupErrorType::MissingTasksFolder,
                    &format!("missing mandatory folder: {:?}", tasks),
                )
            })?,
            workers: canonicalize(PathBuf::from(workers)).map_err(|_e| {
                PupError::with_message(
                    PupErrorType::MissingWorkerFolder,
                    &format!("missing mandatory folder: {:?}", workers),
                )
            })?,
        });
    }

    /// Import an entire environment settings map
    pub fn set_root_environment(&mut self, env: &HashMap<String, String>) {
        for key in env.keys() {
            self.env.insert(key.to_string(), env[key].to_string());
        }
    }

    /// Load a context by 'name string' in the format foo.bar.foobar#version
    pub fn load_task(&self, name: &str) -> Result<(PupTask, PupManifestVersion), PupError> {
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
                let matched = task.manifest.versions.iter().find(|v| {
                    return v.version == version_id;
                });
                if matched.is_none() {
                    return Err(PupError::with_message(
                        PupErrorType::MissingVersion,
                        &format!(
                            "No version matching '{:?}' on {}",
                            version_id,
                            path::display(task.path)
                        ),
                    ));
                }
                matched.unwrap().clone()
            }
            None => {
                if !task.manifest.versions.len() == 0 {
                    return Err(PupError::with_message(
                        PupErrorType::MissingVersion,
                        &format!("No versions available on {}", path::display(task.path)),
                    ));
                }
                task.manifest.versions[task.manifest.versions.len() - 1].clone()
            }
        };

        return Ok((task, version));
    }

    /// Find a worker by the name 'name' in the workers folder, and return a PupWorker for it.
    pub fn load_worker(&self, name: &str) -> Result<PupWorker, PupError> {
        let mut logger = get_logger();

        let attempt1 = join(&self.workers, name);
        logger.log(
            Level::Debug,
            format!("Checking for: {}", path::display(&attempt1)),
        );
        if exists(&attempt1) {
            logger.log(Level::Debug, format!("Found: {}", path::display(&attempt1)));
            return Ok(PupWorker {
                path: attempt1,
                name: String::from(name),
                env: self.env.clone(),
            });
        }

        let attempt2 = join(&self.workers, format!("{}.exe", name));
        logger.log(
            Level::Debug,
            format!("Checking for: {}", path::display(&attempt2)),
        );
        if exists(&attempt2) {
            logger.log(Level::Debug, format!("Found: {}", path::display(&attempt2)));
            return Ok(PupWorker {
                path: attempt2,
                name: String::from(name),
                env: self.env.clone(),
            });
        }

        return Err(PupError::with_message(
            PupErrorType::MissingWorker,
            &format!(
                "Unable to find any worker '{}' in {}",
                name,
                path::display(&self.workers)
            ),
        ));
    }
}

#[cfg(test)]
mod tests {
    use crate::testing::test_fixture;

    #[test]
    fn load_simple_task() {
        let process = test_fixture();
        let (task, version) = process
            .context
            .load_task("tests.actions.setVersion#0.0.2")
            .unwrap();
        assert_eq!(task.manifest.versions.len(), 2);
        assert_eq!(version.version, "0.0.2");
    }

    #[test]
    fn fails_to_load_missing_version() {
        let process = test_fixture();
        let task = process.context.load_task("tests.actions.setVersion#1.0.0");
        assert!(task.is_err());
    }
}

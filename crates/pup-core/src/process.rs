use crate::errors::{PupError, PupErrorType};
use crate::utils::path::{absolute_path, join};

use serde_yaml;

use crate::context::PupContext;
use crate::manifest::PupManifestVersion;
use crate::runner::env::EnvHelper;
use crate::runner::PupRunner;
use crate::task::PupTask;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug)]
pub struct PupProcess {
    /// The root path to the process sequence
    pub path: PathBuf,

    /// The internal context instance
    pub context: PupContext,

    /// The internal manifest instance
    pub manifest: PupProcessManifest,

    /// The ambient external environment overrides, if any
    pub environment_overrides: HashMap<String, String>,
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
                &format!(
                    "Unable to read process manifest: {:?}: {:?}",
                    process_manifest_path,
                    err.description()
                ),
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
    /// Create a process from a path and (optionally) a set of external environmental overrides.
    pub fn load_from<P: AsRef<Path>>(
        path: P,
        env: Option<HashMap<String, String>>,
    ) -> Result<PupProcess, PupError> {
        // Load manifest
        let manifest = PupProcessManifest::try_from(path.as_ref())?;
        let manifest_path: PathBuf = PupProcess::get_manifest_folder(path.as_ref())?;

        // Create context
        let mut context = PupContext::new(
            &join(&manifest_path, &manifest.tasks_path),
            &join(&manifest_path, &manifest.workers_path),
            &manifest_path,
        )?;

        // Render any env variables in the manifest
        PupProcess::render_context_env(&manifest, &mut context, env)?;

        return Ok(PupProcess {
            path: PathBuf::from(path.as_ref()),
            manifest,
            context,
            environment_overrides: HashMap::new(),
        });
    }

    fn get_manifest_folder(manifest_file_path: &Path) -> Result<PathBuf, PupError> {
        let unc_path = manifest_file_path
            .canonicalize()
            .map_err(|err| {
                PupError::with_message(
                    PupErrorType::MissingManifest,
                    &format!("Unable to resolve manifest folder: {:?}", err),
                )
            })?
            .parent()
            .map(|i| PathBuf::from(i))
            .unwrap_or(PathBuf::from("."));

        return absolute_path(unc_path).map_err(|err| {
            PupError::with_message(
                PupErrorType::MissingManifest,
                &format!("Unable to resolve manifest folder: {:?}", err),
            )
        });
    }

    fn render_context_env(
        manifest: &PupProcessManifest,
        context: &mut PupContext,
        overrides: Option<HashMap<String, String>>,
    ) -> Result<(), PupError> {
        let mut env_helper = EnvHelper::new(&context.global_env);
        let mut ambient_params = env_helper.ambient_state().clone();

        // Blat existing values if any override
        match overrides {
            Some(o) => {
                for key in o.keys() {
                    ambient_params.insert(key.to_string(), o[key].to_string());
                }
            }
            None => {}
        };

        let env = env_helper
            .render_existing_keys_from_parent_scope(&manifest.environment, &ambient_params)?;
        context.set_root_environment(&env);
        Ok(())
    }

    pub fn task(&mut self, task: &str) -> Result<(PupTask, PupManifestVersion), PupError> {
        return self.context.load_task(task);
    }

    pub fn runner(&mut self, task: &str) -> Result<PupRunner, PupError> {
        let mut runner = PupRunner::new(&self.context);
        let _ = runner.add(task)?;
        return Ok(runner);
    }
}

#[cfg(test)]
mod tests {
    use super::PupProcess;
    use crate::testing::test_context_process_path;
    use crate::testing::test_fixture;

    #[test]
    fn test_load_from_folder() {
        let sample_process = test_context_process_path();
        let process = PupProcess::load_from(sample_process, None).unwrap();
        assert_eq!(process.manifest.environment["foo"], "bar");
        assert_eq!(
            process.manifest.environment["userthing"],
            "{{EXT_USERNAME}} -> {{EXT_PASSWORD}}"
        );
        let _ = process.context;
    }

    #[test]
    fn test_auto_paths_are_present() {
        let sample_process = test_context_process_path();
        let process = PupProcess::load_from(sample_process, None).unwrap();
        assert!(process.context.global_env.contains_key("MANIFEST_HOME"));
        let _ = process.context;
    }

    #[test]
    fn test_use_manifest_path() {
        let sample_process = test_context_process_path();
        let process = PupProcess::load_from(sample_process, None).unwrap();
        let expected = format!("{}--foo", &process.context.global_env["MANIFEST_HOME"]);
        assert_eq!(process.context.env["uses_manifest"], expected);
        let _ = process.context;
    }

    #[test]
    fn test_load_from_test_scaffold() {
        let mut process = test_fixture();

        // Raw values from manifest
        assert_eq!(process.manifest.environment["foo"], "bar");
        assert_eq!(
            process.manifest.environment["userthing"],
            "{{EXT_USERNAME}} -> {{EXT_PASSWORD}}"
        );

        // Rendered values
        let runner = process.runner("tests.actions.nested").unwrap();
        let action = runner.tasks().children[0].external.take().unwrap();
        assert_eq!(action.env["foo"], "bar");
        assert_eq!(action.env["userthing"], "foouser -> foopass");
    }

    #[test]
    fn test_root_level_tasks() {
        let sample_process = test_context_process_path();
        let process = PupProcess::load_from(sample_process, None).unwrap();
        assert_eq!(process.manifest.tasks.len(), 2);
    }
}

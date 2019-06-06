use ::errors::{PupError, PupErrorType};
use ::utils::path::join;

use ::serde_yaml;

use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::error::Error;
use utils::path::exists;
use utils::path;
use std::collections::HashMap;
use logger::get_logger;
use ::base_logging::Level;


#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct PupManifest {
    /// The name of the action in the root/workers/ folder to execute with this action.
    /// The action "foo" maps to the executable "foo" or "foo.exe" as appropriate.
    pub action: String,

    /// The set of versions available for this action
    pub versions: Vec<PupManifestVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PupManifestVersion {
    /// The version identifier for this version, eg. 0.0.1
    /// The version config file is passed to the action, eg. root/tasks/foo/bar/z/config/0.0.1.json
    pub version: String,

    /// The set of dependencies for this version to execute.
    /// The task foo.bar.z maps to the task in the folder root/tasks/foo/bar/z/
    /// The format should be: path.path.path@version
    #[serde(default)]
    pub steps: Vec<PupManifestStep>,

    /// The path to the folder for this version
    #[serde(skip)]
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PupManifestStep {
    /// The verison identifier for the target step to run.
    /// eg. foo.bar.foo#0.0.1
    pub step: String,

    /// The set of extra env variables just for this step.
    /// Allow handlebar's templates here, eg. FOO_PATH: "{{SOURCE_PATH}}/foo"
    /// Inherit the root env configuration as input variables.
    #[serde(default)]
    pub environment: HashMap<String, String>,

    /// Allow this step to be skipped if some condition is met; if this value is any 'truish' string value skip the step.
    /// Allow handlebar's templates here, eg. skip: "{{SKIP_BUILD_STEP}}"
    /// Inherit the root env configuration as input variables.
    #[serde(default)]
    pub skip: String,

    /// Allow this step to be skipped if some condition is not met; if this value is NOT any 'truish' string value, skip the step.
    /// Allow handlebar's templates here, eg. if: "{{USE_BUILD_STEP}}"
    /// Inherit the root env configuration as input variables.
    /// 
    #[serde(default)]
    #[serde(rename = "if")]
    pub if_marker: String,
}

impl PupManifest {
    pub fn try_from(task_folder: &Path) -> Result<Self, PupError> {
        let manifest_path = join(task_folder, "manifest.yml");
        return PupManifest::read_manifest(task_folder, &manifest_path).map_err(|err| {
            return PupError::with_error(
                PupErrorType::MissingManifest,
                &format!("Unable to read manifest: {}: {:?}", path::display(manifest_path), err.description()),
                err,
            );
        });
    }

    fn read_manifest(task_path: &Path, manifest_path: &Path) -> Result<Self, PupError> {
        let mut fp = File::open(&manifest_path)?;
        let mut raw = String::new();
        fp.read_to_string(&mut raw)?;

        let mut manifest: PupManifest = serde_yaml::from_str(&raw)?;
        manifest.validate(&task_path)?;

        return Ok(manifest);
    }

    /// Check and load all paths in the manifest
    pub fn validate(&mut self, path: &Path) -> Result<(), PupError> {
        let mut logger = get_logger();
        for version in self.versions.iter_mut() {
            let mut version_path = join(path, join("versions", &version.version));
            if !exists(&version_path) {
                // If no versions folder exists, just use the root folder and log a warning.
                logger.log(Level::Debug, format!("No versions folder for: {}, using root: {}", version.version, path::display(path)));
                version_path = PathBuf::from(path);
            }
            version.path = version_path;
        }
        return Ok(());
    }
}
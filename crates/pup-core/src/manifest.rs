use ::errors::{PupError, PupErrorType};
use ::utils::path::join;

use ::serde_yaml;

use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::error::Error;
use utils::path::exists;


#[derive(Debug, Serialize, Deserialize)]
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
    pub steps: Vec<String>,

    /// The path to the folder for this version
    #[serde(skip)]
    pub path: PathBuf,
}

impl PupManifest {
    pub fn try_from(task_folder: &Path) -> Result<Self, PupError> {
        let manifest_path = join(task_folder, "manifest.yml");
        return PupManifest::read_manifest(task_folder, &manifest_path).map_err(|err| {
            return PupError::with_error(
                PupErrorType::MissingManifest,
                &format!("Unable to read manifest: {:?}: {:?}", manifest_path, err.description()),
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
        for version in self.versions.iter_mut() {
            let version_path = join(path, join("config", &version.version));
            if !exists(&version_path) {
                return Err(PupError::with_message(
                    PupErrorType::MissingVersionFolder,
                    &format!("Missing version directory: {:?}", &version_path),
                ));
            }
            version.path = PathBuf::from(path);
        }
        return Ok(());
    }
}
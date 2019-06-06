use std::path::Path;
use std::fs::File;
use pup_worker::errors::PupWorkerError;
use serde_yaml;
use std::io::Read;
use pup_worker::logger::get_logger;
use pup_worker::logger::Level;
use pup_worker::utils::path;

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchManifest {
    pub tasks: Vec<PatchTask>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchTask {
    /// The path to the file to patch
    pub input: String,

    /// Where to write the patched file
    pub output: String,

    /// The patch mode to apply
    pub mode: PatchMode,

    /// The set of patch mode flags
    #[serde(default)]
    pub flags: Vec<PatchModeFlag>,

    /// The set of patches to apply
    #[serde(default)]
    pub patch: Vec<PatchTaskItem>,

    /// Should this step be skipped? Render the value using the env context, and if it is truish,
    /// skip this step.
    #[serde(default)]
    pub skip: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchTaskItem {
    /// The match pattern to match on
    pub pattern: String,

    /// If the number of submatches in the pattern to not match the number of matches in the values,
    /// still perform a partial match and replace what we can. If this is false (default), a mismatch
    /// will trigger an error.
    #[serde(default)]
    pub partial: bool,

    /// The value to replace with; apply env content formatting.
    #[serde(default)]
    pub values: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum PatchModeFlag {
    /// Skip without an error if the input file is missing.
    SkipIfInputMissing,

    /// Skip without an error if the output file already exists.
    SkipIfOutputExists,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum PatchMode {
    Copy,
    Regex,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchChange {
    /// The target to replace
    target: String,

    /// The value to replace it with
    value: String,
}

impl PatchManifest {
    pub fn try_from<P: AsRef<Path>>(task_folder: P) -> Result<Self, PupWorkerError> {
        let mut logger = get_logger()?;
        let manifest_path = task_folder.as_ref().join("main.yml");
        logger.log(Level::Debug, format!("Trying manifest path: {}", path::display(&manifest_path)));
        return Self::read_manifest(&manifest_path);
    }

    fn read_manifest(manifest_path: &Path) -> Result<Self, PupWorkerError> {
        let mut fp = PupWorkerError::wrap(File::open(&manifest_path))?;
        let mut raw = String::new();
        PupWorkerError::wrap(fp.read_to_string(&mut raw))?;
        return PupWorkerError::wrap(serde_yaml::from_str(&raw));
    }
}
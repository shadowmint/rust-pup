use std::path::Path;
use std::fs::File;
use std::io::Read;
use ::serde_yaml;

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskManifest {
    pub tasks: Vec<TaskItem>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskItem {
    /// Some description of the task.
    #[serde(default)]
    pub info: String,

    /// The path to execute in.
    #[serde(default)]
    pub path: String,
    
    /// The binary to execute
    pub task: String,

    /// A set of variants if the primary task can't be found. eg. foo.exe for windows.
    #[serde(default = "TaskManifest::default_task_variants")]
    pub task_variants: Vec<String>,

    /// The argument string template (handlebars)
    pub args: Vec<String>,

    /// Save the command output to a file
    /// Doesn't work with 'dont_wait'
    #[serde(default)]
    pub output: String,

    /// Allow failure; eg. if a move / clone / etc fails then allow it to continue
    #[serde(default = "default_failure_mode")]
    #[serde(rename = "continueOnFailure")]
    pub continue_on_failure: bool,

    /// Disconnect the process and return zero if it spawns without waiting
    #[serde(default)]
    #[serde(rename = "dontWait")]
    pub dont_wait: bool
}

fn default_failure_mode() -> bool {
    false
}

impl TaskManifest {
    pub fn try_from(task_folder: &Path) -> Self {
        let manifest_path = task_folder.join("main.yml");
        return Self::read_manifest(&manifest_path);
    }

    fn read_manifest(manifest_path: &Path) -> Self {
        let mut fp = File::open(&manifest_path).unwrap();
        let mut raw = String::new();
        fp.read_to_string(&mut raw).unwrap();
        return serde_yaml::from_str(&raw).unwrap();
    }

    fn default_task_variants() -> Vec<String> {
        Vec::new()
    }
}
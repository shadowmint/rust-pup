use std::path::PathBuf;
use std::collections::HashMap;

/// An external binary command to execute for an action.
pub struct PupWorker {
    /// Full path to the executable
    pub path: PathBuf,

    /// The name of the executable
    pub name: String,

    /// The set of environment variable to use
    pub env: HashMap<String, String>,
}

/// The result of executing a PupWorker
pub struct PupWorkerResult {}
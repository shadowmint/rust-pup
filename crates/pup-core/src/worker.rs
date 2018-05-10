use std::path::PathBuf;

/// An external binary command to execute for an action.
pub struct PupWorker {
    
    /// Full path to the executable
    pub path: PathBuf,
    
    /// The name of the executable
    pub name: String,
}

/// The result of executing a PupWorker
pub struct PupWorkerResult {}
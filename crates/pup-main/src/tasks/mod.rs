use ::pup_core::PupError;
use ::PupArg;
use std::collections::HashMap;

pub mod list_available_tasks;

pub trait PupTaskRunner {
    /// Check the runner can actually run, and save required state.
    fn prepare(&mut self, args: HashMap<PupArg, String>) -> Result<(), PupError>;

    /// Actually run.
    fn run(&mut self) -> Result<u32, PupError>;
}
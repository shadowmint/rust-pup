use ::pup_core::PupError;
use ::PupArg;
use ::base_logging::Logger;
use std::collections::HashMap;

pub trait PupTaskRunner {
    /// Check the runner can actually run, and save required state.
    fn prepare(&mut self, args: HashMap<PupArg, String>) -> Result<(), PupError>;

    /// Is ready to run?
    fn ready(&self) -> bool;

    /// Actually run.
    fn run(&mut self, logger: &mut Logger) -> Result<u32, PupError>;
}
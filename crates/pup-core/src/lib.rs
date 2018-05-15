#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate lazy_static;

extern crate dunce;
extern crate serde_yaml;
extern crate base_logging;
extern crate time;

mod context;
mod manifest;
mod task;
mod errors;
mod runner;
mod worker;
mod process;

pub mod utils;
pub mod logger;
pub mod testing;

pub use context::PupContext;
pub use runner::PupActionOptions;
pub use errors::{PupError, PupErrorType};
pub use process::PupProcess;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

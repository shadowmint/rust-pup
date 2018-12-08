#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate lazy_static;

extern crate dunce;
extern crate serde_yaml;
extern crate base_logging;
extern crate time;
extern crate handlebars;

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

pub use crate::context::PupContext;
pub use crate::runner::PupActionOptions;
pub use crate::errors::{PupError, PupErrorType};
pub use crate::process::PupProcess;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

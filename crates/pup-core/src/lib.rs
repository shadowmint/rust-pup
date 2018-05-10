#[macro_use]
extern crate serde_derive;

extern crate serde_yaml;
extern crate base_logging;

mod context;
mod manifest;
mod task;
mod errors;
mod utils;
mod runner;
mod worker;
mod logger;

#[cfg(test)]
mod fixtures;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

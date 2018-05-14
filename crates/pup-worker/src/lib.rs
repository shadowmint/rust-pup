#[macro_use]
extern crate lazy_static;

extern crate serde_yaml;
extern crate base_logging;
extern crate time;

pub mod errors;
pub mod utils;
pub mod logger;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

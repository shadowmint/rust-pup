#[cfg(test)]
#[allow(dead_code)]
#[macro_use]
extern crate log;

#[allow(dead_code)]
#[macro_use]
extern crate serde_derive;

#[macro_use]
mod core;

mod infrastructure;

pub mod formatters;
pub mod loggables;
pub mod loggers;

pub use core::traits::Log;
pub use core::traits::Loggable;
pub use core::traits::LogFormatter;
pub use core::logger::Logger;
pub use core::level::Level;
pub use core::logger_ref::LoggerRef;
pub use loggables::Record;
pub use infrastructure::interop::setup_standard_logger;

#[cfg(test)]
mod tests {
    use ::Log;
    use ::Logger;
    use ::Level;

    pub struct CustomLogger {}

    impl Log for CustomLogger {
        fn log(&mut self, message: &str) {
            println!("CustomLogger: {}", message);
        }
    }

    #[test]
    fn test_custom_logger() {
        let mut logger = Logger::new().with(CustomLogger {});
        logger.log(Level::Info, format_log!("Hello {}", "Word"));
        logger.log(Level::Info, format_log!({"Hello {}", "Word"}, {"extra" => "property"}));
    }
}

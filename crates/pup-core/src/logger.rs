use ::base_logging::Logger;
use ::base_logging::Level;
use ::base_logging::LogFormatter;
use ::base_logging::loggers::ConsoleLogger;
use std::collections::HashMap;
use ::time::Tm;

struct PupFormatter {}

impl LogFormatter for PupFormatter {
    fn log_format(&self, level: Level, timestamp: Tm, message: Option<&str>, properties: Option<HashMap<&str, &str>>) -> String {
        match message {
            Some(m) => String::from(m),
            None => String::new()
        }
    }
}

pub fn get_logger() -> Logger {
    let logger = Logger::new().with_format(PupFormatter {}).with(ConsoleLogger::new());
    return logger;
}
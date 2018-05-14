use ::base_logging::Logger;
use ::base_logging::Level;
use ::base_logging::LogFormatter;
use ::base_logging::loggers::ConsoleLogger;
use std::collections::HashMap;
use ::time::Tm;
use std::sync::Mutex;

struct PupFormatter {}

lazy_static! {
    static ref LEVEL: Mutex<Level> = Mutex::new(Level::Info);
}

impl LogFormatter for PupFormatter {
    fn log_format(&self, level: Level, _timestamp: Tm, message: Option<&str>, _properties: Option<HashMap<&str, &str>>) -> String {
        if level <= *LEVEL.lock().unwrap() {
            return match message {
                Some(m) => String::from(m),
                None => String::new()
            };
        };
        return String::new();
    }
}

pub fn get_logger() -> Logger {
    let logger = Logger::new().with_format(PupFormatter {}).with(ConsoleLogger::new());
    return logger;
}

pub fn set_logger_level(level: Level) {
    let mut level_ref = LEVEL.lock().unwrap();
    *level_ref = level;
}
use ::base_logging::Logger;
use ::base_logging::loggers::ConsoleLogger;

pub fn get_logger() -> Logger {
    let logger = Logger::new().with(ConsoleLogger::new());
    return logger;
}
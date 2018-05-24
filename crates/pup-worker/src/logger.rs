use ::base_logging::Logger;
pub use ::base_logging::Level;
use ::base_logging::loggers::ConsoleLogger;
use std::sync::Mutex;
use errors::PupWorkerError;
use std::borrow::Borrow;
use base_logging::formatters::DefaultFormatter;

/// The global logging configuration
pub struct LogConfig {
    factory: Option<fn() -> Logger>,
    level: Level,
    prefix: String,
}

/// The global logging configuration instance
lazy_static! {
    static ref CONFIG: Mutex<LogConfig> = Mutex::new(LogConfig {
        level: Level::Info,
        prefix: "pup-worker".to_string(),
        factory: None
    });
}

pub fn configure_console_logging(prefix: &str, level: Level) -> Result<(), PupWorkerError> {
    let mut config_ref = PupWorkerError::wrap(CONFIG.lock())?;
    config_ref.level = level;
    config_ref.prefix = prefix.to_string();
    Ok(())
}

pub fn get_logger() -> Result<Logger, PupWorkerError> {
    let level_ref = PupWorkerError::wrap(CONFIG.lock())?;
    let logger = match level_ref.factory {
        Some(f) => f(),
        None => default_logger(level_ref.borrow())
    };
    return Ok(logger);
}

pub fn set_logger(factory: fn() -> Logger) -> Result<(), PupWorkerError> {
    let mut level_ref = PupWorkerError::wrap(CONFIG.lock())?;
    level_ref.factory = Some(factory);
    Ok(())
}

pub fn default_logger(config: &LogConfig) -> Logger {
    return Logger::new()
        .with_level(config.level)
        .with_format(DefaultFormatter::new().with_prefix(&config.prefix))
        .with(ConsoleLogger::new());
}

pub fn default_formatter(prefix: &str) -> DefaultFormatter {
    return DefaultFormatter::new().with_prefix(prefix);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_logger() {
        set_logger_level(Level::Debug).unwrap();
        let mut l = get_logger().unwrap();
        l.log(Level::Debug, "Debug!");
        l.log(Level::Info, "Info");
    }

    #[test]
    fn test_custom_logger() {
        set_logger(custom_logger).unwrap();
        set_logger_level(Level::Info).unwrap();
        let mut l = get_logger().unwrap();
        l.log(Level::Debug, "Debug!");
        l.log(Level::Info, "Info");
    }

    fn custom_logger() -> Logger {
        return Logger::new().with(ConsoleLogger::new());
    }
}

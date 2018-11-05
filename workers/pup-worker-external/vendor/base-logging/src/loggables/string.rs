use core::traits::Loggable;
use std::collections::HashMap;

impl Loggable for String {
    fn log_message(&self) -> Option<&str> {
        Some(&self)
    }
    fn log_properties(&self) -> Option<HashMap<&str, &str>> {
        None
    }
}

impl<'a> Loggable for &'static str {
    fn log_message(&self) -> Option<&str> {
        Some(self)
    }
    fn log_properties(&self) -> Option<HashMap<&str, &str>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use ::Logger;
    use loggers::MockLogger;
    use ::Level;

    #[test]
    fn test_str_is_loggable() {
        let mut logger = Logger::new().with(MockLogger::new());
        logger.log(Level::Info, "Hello World");
        logger.log(Level::Info, String::from("Hello World"));
    }

    #[test]
    fn test_format_is_loggable() {
        let mut logger = Logger::new().with(MockLogger::new());
        logger.log(Level::Info, format!("Hello {}", "World"));
    }
}

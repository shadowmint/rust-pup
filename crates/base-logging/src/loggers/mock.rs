use core::traits::Log;
use infrastructure::stdout::debug;

/// A basic mokck of Logger for testing purposes.
/// To enable logging in test mode, use `debug()`.
pub struct MockLogger {
    enable_debug: bool
}

impl MockLogger {
    pub fn new() -> MockLogger {
        return MockLogger {
            enable_debug: false
        };
    }

    /// Enable debug mode, where log messages are forced to stdout.
    pub fn debug(mut self) -> MockLogger {
        self.enable_debug = true;
        return self;
    }
}

impl Log for MockLogger {
    fn log(&mut self, message: &str) {
        if self.enable_debug {
            debug(&format!("{}", message));
        }
    }
}

#[cfg(test)]
mod tests {
    use ::Logger;
    use loggers::MockLogger;
    use ::Level;

    #[test]
    fn test_mock_logger() {
        let mut logger = Logger::new().with(MockLogger::new());
        logger.log(Level::Info, "Hello World");
    }
}

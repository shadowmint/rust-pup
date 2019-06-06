use ::{Logger, Log, Loggable, Level, LogFormatter};
use std::sync::{Arc, Mutex};

/// Logger ref is a convenient Arc<Mutex<Logger>> to allow use in immutable situations.
pub struct LoggerRef {
    logger: Arc<Mutex<Option<Logger>>>
}

impl LoggerRef {
    pub fn new() -> LoggerRef {
        return LoggerRef {
            logger: Arc::new(Mutex::new(Some(Logger::new())))
        };
    }

    pub fn with<T>(self, logger: T) -> LoggerRef where T: Log + Send + 'static {
        match self.logger.lock() {
            Ok(mut logger_ref) => {
                *logger_ref = Some(logger_ref.take().unwrap().with(logger));
            }
            Err(_) => {}
        }
        return self;
    }

    pub fn with_format<T>(self, formatter: T) -> LoggerRef where T: LogFormatter + Send + 'static {
        match self.logger.lock() {
            Ok(mut logger_ref) => {
                *logger_ref = Some(logger_ref.take().unwrap().with_format(formatter));
            }
            Err(_) => {}
        }
        return self;
    }

    pub fn log<'b, 'c, T>(&'c self, level: Level, message: T) where T: Loggable + 'b {
        match self.logger.lock() {
            Ok(mut logger_ref) => {
                logger_ref.as_mut().unwrap().log(level, message);
            }
            Err(_) => {}
        }
    }
}

impl Clone for LoggerRef {
    fn clone(&self) -> Self {
        return LoggerRef {
            logger: self.logger.clone()
        };
    }
}

#[cfg(test)]
mod tests {
    use ::{LoggerRef, Level};
    use formatters::JsonFormatter;
    use loggers::MockLogger;
    use std::thread;

    #[test]
    fn test_logger_ref_from_immutable_ref_cross_thread() {
        let logger = LoggerRef::new().with(MockLogger::new()).with_format(JsonFormatter::new());
        let copy = logger.clone();
        logger.log(Level::Info, "Hello World");
        let  _ = thread::spawn(move || {
            copy.log(Level::Info, "Hello World");
        }).join();
    }
}

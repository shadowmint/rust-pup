extern crate time;

use core::traits::Log;
use core::traits::Loggable;
use core::traits::LogFormatter;
use core::level::Level;
use ::formatters::DefaultFormatter;
use std::collections::HashMap;

pub struct Logger {
    loggers: Vec<Box<Log + Send>>,
    format: Box<LogFormatter + Send>,
    level: Level,
}

impl<'a> Logger {
    pub fn new() -> Logger {
        return Logger {
            loggers: Vec::new(),
            format: Box::new(DefaultFormatter::new()),
            level: Level::Warn,
        };
    }

    pub fn with<T>(mut self, logger: T) -> Logger where T: Log + Send + 'static {
        self.loggers.push(Box::new(logger) as Box<Log + Send>);
        return self;
    }

    pub fn with_format<T>(mut self, formatter: T) -> Logger where T: LogFormatter + Send + 'static {
        self.format = Box::new(formatter);
        return self;
    }

    pub fn with_level(mut self, min_level: Level) -> Logger {
        self.level = min_level;
        return self;
    }

    pub fn log<'b, 'c, T>(&'c mut self, level: Level, loggable: T) where T: Loggable + 'b {
        if level > self.level {
            return;
        }
        let now = time::now();
        let message = loggable.log_message();
        let properties = loggable.log_properties();
        if !self.is_loggable(&message, &properties) {
            return;
        }
        let raw = self.format.log_format(level, now, message, properties);
        if raw.len() == 0 {
            return;
        }
        for target in self.loggers.iter_mut() {
            target.log(&raw);
        }
    }

    fn is_loggable(&self, message: &Option<&str>, props: &Option<HashMap<&str, &str>>) -> bool {
        match message {
            Some(s) => {
                if s.len() > 0 {
                    return true;
                }
            }
            None => {}
        };
        match props {
            Some(p) => {
                if p.keys().len() > 0 {
                    return true;
                }
            }
            None => {}
        };
        return false;
    }
}

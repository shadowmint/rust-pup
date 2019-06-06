extern crate time;

use std::collections::HashMap;
use core::level::Level;
use self::time::Tm;

pub trait Log {
    fn log(&mut self, message: &str);
}

pub trait Loggable {
    fn log_message(&self) -> Option<&str>;
    fn log_properties(&self) -> Option<HashMap<&str, &str>>;
}

pub trait LogFormatter {
    fn log_format(&self, level: Level, timestamp: Tm, message: Option<&str>, properties: Option<HashMap<&str, &str>>) -> String;
}
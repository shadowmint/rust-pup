use ::base_logging::Logger;
use ::base_logging::Level;
use ::base_logging::LogFormatter;
use ::base_logging::loggers::ConsoleLogger;
use std::collections::HashMap;
use ::time::Tm;
use ::time;
use std::sync::Mutex;

struct PupFormatter {}

lazy_static! {
    static ref LEVEL: Mutex<Level> = Mutex::new(Level::Info);
}

impl LogFormatter for PupFormatter {
    fn log_format(&self, level: Level, timestamp: Tm, message: Option<&str>, properties: Option<HashMap<&str, &str>>) -> String {
        if level == Level::Debug {
            let timestring = match time::strftime("%b %d %H:%M:%S", &timestamp) {
                Ok(i) => i,
                Err(_) => String::from("")
            };
            return format!("{} {:?} - {}", timestring, level, self.combine(message, properties));
        } else {
            return match message {
                Some(m) => String::from(m),
                None => String::new()
            };
        }
    }
}

impl PupFormatter {
    fn combine(&self, message: Option<&str>, properties: Option<HashMap<&str, &str>>) -> String {
        let mut rtn = String::new();
        match message {
            Some(msg) => {
                rtn.push_str(msg);
            }
            None => {}
        };
        match properties {
            Some(props) => {
                for (key, value) in props.iter() {
                    rtn.push_str(&format!(" {}:{}", key, value));
                }
            }
            None => {}
        }
        return rtn;
    }
}

pub fn get_logger() -> Logger {
    let mut logger = Logger::new().with_format(PupFormatter {}).with(ConsoleLogger::new());
    match LEVEL.lock() {
        Ok(l) => {
            logger = logger.with_level(*l);
        }
        Err(e) => {
            println!("Failed to get log level information: {:?}", e);
        }
    }
    return logger;
}

pub fn set_logger_level(level: Level) {
    let mut level_ref = LEVEL.lock().unwrap();
    *level_ref = level;
}
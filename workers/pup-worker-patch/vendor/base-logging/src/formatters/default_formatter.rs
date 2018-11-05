extern crate time;

use std::collections::HashMap;
use ::{Level, LogFormatter};
use self::time::Tm;

pub struct DefaultFormatter {
    prefix: Option<String>
}

impl DefaultFormatter {
    pub fn new() -> DefaultFormatter {
        return DefaultFormatter {
            prefix: None
        };
    }

    pub fn with_prefix(mut self, prefix: &str) -> DefaultFormatter {
        self.prefix = Some(prefix.to_string());
        self
    }

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

impl LogFormatter for DefaultFormatter {
    fn log_format(&self, level: Level, timestamp: Tm, message: Option<&str>, properties: Option<HashMap<&str, &str>>) -> String {
        let timestring = match time::strftime("%b %d %H:%M:%S", &timestamp) {
            Ok(i) => i,
            Err(_) => String::from("")
        };
        return match self.prefix {
            Some(ref p) => format!("{}: {} {:?} - {}", p, timestring, level, self.combine(message, properties)),
            None => format!("{} {:?} - {}", timestring, level, self.combine(message, properties))
        };
    }
}
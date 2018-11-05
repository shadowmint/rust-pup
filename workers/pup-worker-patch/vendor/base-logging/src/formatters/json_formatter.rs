extern crate time;
extern crate serde;
extern crate serde_json;

use std::collections::HashMap;
use ::{Level, LogFormatter};
use self::time::Tm;

#[derive(Serialize, Deserialize)]
struct JsonLogRecord {
    timestamp: String,
    message: String,
    level: String,
    properties: HashMap<String, String>,
}

pub struct JsonFormatter {}

impl JsonFormatter {
    pub fn new() -> JsonFormatter {
        return JsonFormatter {};
    }
}

impl LogFormatter for JsonFormatter {
    fn log_format(&self, level: Level, timestamp: Tm, message: Option<&str>, properties: Option<HashMap<&str, &str>>) -> String {
        let timestamp = match time::strftime("%b %d %H:%M:%S", &timestamp) {
            Ok(i) => i,
            Err(_) => String::from("")
        };

        let mut record = JsonLogRecord {
            timestamp,
            level: format!("{:?}", level),
            message: match message {
                Some(m) => String::from(m),
                None => String::from(""),
            },
            properties: HashMap::<String, String>::new(),
        };

        match properties {
            Some(p) => {
                for (key, value) in p.iter() {
                    record.properties.insert(String::from(*key), String::from(*value));
                }
            }
            None => {}
        }

        return match serde_json::to_string(&record) {
            Ok(r) => r,
            Err(_) => String::from("Failed to serialize json record")
        };
    }
}


#[cfg(test)]
mod tests {
    use ::{Logger, Level};
    use ::formatters::JsonFormatter;
    use loggers::MockLogger;

    #[test]
    fn test_json_formatter() {
        let mut logger = Logger::new().with_format(JsonFormatter::new()).with(MockLogger::new());
        logger.log(Level::Info, format_log!({"Hello World {}", 99}, {"extra" => "value"}));
    }
}
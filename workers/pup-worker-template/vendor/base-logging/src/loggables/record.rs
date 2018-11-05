use std::collections::HashMap;
use ::Loggable;

pub struct Record {
    message: String,
    properties: HashMap<&'static str, String>,
}

impl Record {
    pub fn new(message: &str) -> Record {
        return Record {
            message: String::from(message),
            properties: HashMap::<&'static str, String>::new(),
        };
    }

    pub fn property(mut self, property: &'static str, value: &str) -> Record {
        self.properties.insert(property, String::from(value));
        return self;
    }
}

impl Loggable for Record {
    fn log_message(&self) -> Option<&str> {
        Some(&self.message)
    }

    fn log_properties(&self) -> Option<HashMap<&str, &str>> {
        let mut props = HashMap::<&str, &str>::new();
        for (key, value) in self.properties.iter() {
            props.insert(key, value);
        }
        return Some(props);
    }
}


#[cfg(test)]
mod tests {
    use ::{Logger, Level, Record};
    use loggers::MockLogger;

    #[test]
    fn test_record_is_loggable() {
        let message = Record::new("Hello World").property("Foo", "Bar").property("line", "99");
        let mut logger = Logger::new().with(MockLogger::new());
        logger.log(Level::Info, message);
    }

    #[test]
    fn test_record_is_loggable_with_macro() {
        let mut logger = Logger::new().with(MockLogger::new());
        
        logger.log(Level::Info, format_log!("Hello World {}", 98));

        logger.log(Level::Info, format_log!("Hello World {}", 99)
            .property("one", "two")
            .property("three", &format!("1 {}", 2)));

        logger.log(Level::Info, format_log!({"Hello World {}", 99}, {"extra" => "value"}));
    }
}

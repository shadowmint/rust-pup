use core::traits::Loggable;
use std::collections::HashMap;

impl Loggable for HashMap<&'static str, &'static str> {
    fn log_message(&self) -> Option<&str> {
        None
    }
    fn log_properties(&self) -> Option<HashMap<&str, &str>> {
        Some(self.clone())
    }
}

impl Loggable for HashMap<&'static str, String> {
    fn log_message(&self) -> Option<&str> {
        None
    }
    fn log_properties(&self) -> Option<HashMap<&str, &str>> {
        let mut props = HashMap::<&str, &str>::new();
        for (key, value) in self.iter() {
            props.insert(key, value);
        }
        return Some(props);
    }
}

impl Loggable for HashMap<String, String> {
    fn log_message(&self) -> Option<&str> {
        None
    }
    fn log_properties(&self) -> Option<HashMap<&str, &str>> {
        let mut props = HashMap::<&str, &str>::new();
        for (key, value) in self.iter() {
            props.insert(key, value);
        }
        return Some(props);
    }
}

#[cfg(test)]
mod tests {
    use ::Logger;
    use loggers::MockLogger;
    use std::collections::HashMap;
    use ::Level;

    #[test]
    fn test_hashmap_is_loggable() {
        let mut message = HashMap::<&'static str, &'static str>::new();
        message.insert("Hello", "World");
        message.insert("V1", "V2");

        let mut message2 = HashMap::<&'static str, String>::new();
        message2.insert("Hello", format!("Hello {}", "World"));
        message2.insert("V1", format!("Hello {}", "V1"));

        let mut message3 = HashMap::<String, String>::new();
        message3.insert(format!("Hello {}", "World"), format!("Hello {}", "World"));
        message3.insert(format!("Hello {}", "V1"), format!("Hello {}", "V1"));

        let mut logger = Logger::new().with(MockLogger::new());
        logger.log(Level::Info, message);
        logger.log(Level::Warn, message2);
        logger.log(Level::Debug, message3);
    }
}

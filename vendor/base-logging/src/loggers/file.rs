use ::Log;

pub struct FileLogger {
}

impl FileLogger {
    pub fn new() -> FileLogger {
        return FileLogger {};
    }

    pub fn with_target(self, _: &str) -> FileLogger {
        self
    }
}

impl Log for FileLogger {
    fn log(&mut self, _: &str) {}
}

#[cfg(test)]
mod tests {
    use ::Logger;
    use ::Level;
    use loggers::FileLogger;

    #[test]
    fn test_mock_logger() {
        let file_target = FileLogger::new().with_target("log.txt");
        let mut logger = Logger::new().with(file_target);
        logger.log(Level::Info, "Test console logger");
    }
}

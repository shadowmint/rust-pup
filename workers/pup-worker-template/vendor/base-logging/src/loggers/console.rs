use core::traits::Log;
use infrastructure::stdout::stderr;
use std::io::Write;

pub struct ConsoleLogger {}

impl ConsoleLogger {
    pub fn new() -> ConsoleLogger {
        return ConsoleLogger {};
    }
}

fn write(message: &str) {
    if message.len() == 0 {
        return;
    }
    let mut guard = stderr();
    let _ = guard.write(message.as_bytes());
    let _ = guard.write(&['\n' as u8]);
}

impl Log for ConsoleLogger {
    fn log(&mut self, message: &str) {
        write(&format!("{}", message));
    }
}

#[cfg(test)]
mod tests {
    use ::Logger;
    use ::Level;
    use loggers::ConsoleLogger;

    #[test]
    fn test_mock_logger() {
        let mut logger = Logger::new().with(ConsoleLogger::new());
        logger.log(Level::Info, "Test console logger");
    }
}

#[macro_use]
extern crate base_logging;

use base_logging::Logger;
use base_logging::loggers::ConsoleLogger;
use base_logging::Level;
use base_logging::formatters::DefaultFormatter;

#[test]
fn test_simple_logger() {
    let mut logger = Logger::new()
        .with(ConsoleLogger::new())
        .with_format(DefaultFormatter::new())
        .with_level(Level::Info);
    
    logger.log(Level::Info, "");
    logger.log(Level::Info, format_log!("Hello {}", "Word"));
    logger.log(Level::Debug, format_log!({"Hello {}", "Word"}, {"extra" => "property"}));
}
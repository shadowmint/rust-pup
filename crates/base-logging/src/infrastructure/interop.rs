extern crate log;

use self::log::{Log, Metadata, Record, set_logger, set_max_level, LevelFilter};
use self::log::Level as LogLevel;
use core::logger::Logger;
use core::level::Level;
use std::sync::{ONCE_INIT, Once, Mutex, MutexGuard, LockResult};
use std::mem;

static LOGGER_DUMMY: DummyLogger = DummyLogger {};

struct DummyLogger {}

struct SingletonLogger {
    inner: Mutex<Option<Logger>>
}

fn logger_singleton() -> LockResult<MutexGuard<'static, Option<Logger>>> {
    static mut LOGGER_SINGLETON: *const SingletonLogger = 0 as *const SingletonLogger;
    static ONCE: Once = ONCE_INIT;

    unsafe {
        ONCE.call_once(|| {
            let singleton = SingletonLogger {
                inner: Mutex::new(None),
            };

            LOGGER_SINGLETON = mem::transmute(Box::new(singleton));
        });

        return (*LOGGER_SINGLETON).inner.lock();
    }
}

pub fn from_log_level(level: LogLevel) -> Level {
    match level {
        i if i == LogLevel::Error => Level::Error,
        i if i == LogLevel::Warn => Level::Warn,
        i if i == LogLevel::Info => Level::Info,
        i if i == LogLevel::Debug => Level::Debug,
        i if i == LogLevel::Trace => Level::Trace,
        _ => Level::Trace
    }
}

pub fn setup_standard_logger(logger: Logger) -> bool {
    match logger_singleton() {
        Ok(mut inner) => {
            *inner = Some(logger);
            match set_logger(&LOGGER_DUMMY) {
                Ok(_) => {}
                Err(_) => return false
            };
            set_max_level(LevelFilter::Trace);
        }
        Err(_) => return false
    };
    return true;
}

impl Log for DummyLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        return true;
    }

    // Notice this always uses the singleton instance, because Log requires a read-only
    // instance, but we require a mutable instance.
    fn log(&self, record: &Record) {
        match logger_singleton() {
            Ok(mut inner) => {
                match *inner {
                    Some(ref mut logger) => {
                        let level = from_log_level(record.level());
                        let target = format!("{}", record.args());
                        logger.log(level, target);
                    }
                    None => {}
                }
            }
            Err(_) => {}
        }
    }

    fn flush(&self) {}
}


#[cfg(test)]
mod tests {
    use ::{Logger, setup_standard_logger};
    use ::loggers::MockLogger;

    #[test]
    fn test_interop() {
        let logger = Logger::new().with(MockLogger::new());
        assert!(setup_standard_logger(logger));
        info!("Test interop logging: {}", "World");
    }
}

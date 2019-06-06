mod mock;
mod file;
mod console;

pub use self::mock::MockLogger;
pub use self::console::ConsoleLogger;
pub use self::file::FileLogger;
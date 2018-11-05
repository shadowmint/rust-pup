extern crate pup_worker;

use pup_worker::logger::Level;
use pup_worker::logger::get_logger;
use pup_worker::errors::PupWorkerError;

#[test]
fn test_simple_worker() {
    test_simple_worker_result().unwrap();
}

fn test_simple_worker_result() -> Result<(), PupWorkerError> {
    let mut logger = get_logger()?;
    logger.log(Level::Debug, "hello world");
    Ok(())
}

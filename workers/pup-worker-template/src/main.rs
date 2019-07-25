#[macro_use]
extern crate serde_derive;

extern crate pup_worker;
extern crate base_logging;
extern crate handlebars;
extern crate serde_yaml;
extern crate getopts;
extern crate regex;

mod logger;
mod manifest;
mod actions;
mod arguments;

use crate::logger::{configure_logging, get_logger};
use pup_worker::errors::PupWorkerError;
use pup_worker::logger::Level;
use crate::manifest::Manifest;
use std::process;
use std::error::Error;
use crate::actions::process_manifest;
use crate::arguments::process_args;
use crate::arguments::Arguments;

fn main() {
    match process_args() {
        Ok(args) => {
            // Setup logging
            if args.verbose_logging {
                let results = configure_logging(Level::Debug);
                if results.is_err() {
                    print_err_and_exit(results.err().unwrap());
                }
            } else {
                let results = configure_logging(Level::Info);
                if results.is_err() {
                    print_err_and_exit(results.err().unwrap());
                }
            }

            // Run worker
            let results = run(args);
            if results.is_err() {
                print_err_and_exit(results.err().unwrap());
            }
        }
        Err(err) => print_err_and_exit(err)
    }
}

fn print_err_and_exit(err: PupWorkerError) {
    println!("{}", err.description());
    process::exit(1);
}

fn run(args: Arguments) -> Result<(), PupWorkerError> {
    let mut logger = get_logger()?;

    logger.log(Level::Debug, "Reading manifest");
    let manifest = Manifest::try_from(args.manifest_path)?;

    logger.log(Level::Debug, "Processing manifest");
    process_manifest(manifest, &mut logger)?;

    logger.log(Level::Debug, "Completed");
    Ok(())
}



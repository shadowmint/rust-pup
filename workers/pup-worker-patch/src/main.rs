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
mod patchers;
mod patch;

use logger::{configure_logging, get_logger};
use pup_worker::errors::PupWorkerError;
use pup_worker::logger::Level;
use manifest::PatchManifest;
use std::env;
use getopts::Options;
use std::process;
use std::error::Error;
use patch::process_patch_task;
use pup_worker::errors::PupWorkerErrorType;

fn main() -> Result<(), PupWorkerError> {
    process_args()?;

    let mut logger = get_logger()?;

    logger.log(Level::Debug, "Reading manifest");
    let mut manifest = PatchManifest::try_from(".")?;

    logger.log(Level::Debug, format!("Found {} files to patch", manifest.tasks.len()));
    for task in manifest.tasks.iter_mut() {
        match process_patch_task(task, &mut logger) {
            Err(e) => {
                // Skips are non-fatal
                if e.error_type != PupWorkerErrorType::SkipTask {
                    return Err(e);
                }
            }
            Ok(_) => {}
        }
    }

    Ok(())
}

fn process_args() -> Result<(), PupWorkerError> {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "show help message");
    opts.optflag("v", "verbose", "use verbose logging");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => {
            print_err_bad_usage(&format!("{}", f.description()), &program, opts);
            process::exit(1);
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        process::exit(0);
    }

    if matches.opt_present("v") {
        configure_logging(Level::Debug)?;
        let mut logger = get_logger()?;
        logger.log(Level::Debug, "Using verbose logging");
    } else {
        configure_logging(Level::Info)?;
    }

    Ok(())
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn print_err_bad_usage(error: &str, program: &str, opts: Options) {
    println!("Error: {}", error);
    print_usage(program, opts);
}

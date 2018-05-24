extern crate getopts;
extern crate pup_main;

use std::env;
use getopts::Options;
use pup_main::{pup_enable_debug, pup_main, PupArg, PupTask};
use std::collections::HashMap;
use std::process;
use std::error::Error;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]\n     : If FILE is ommitted, 'manifest.yml' will be used.", program);
    
    print!("{}", opts.usage(&brief));
}

fn err_bad_usage(error: &str, program: &str, opts: Options) {
    println!("Error: {}", error);
    print_usage(program, opts);
}

fn err_failure(error: &str) {
    println!("Error: {}", error);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("t", "task", "the id of the task to run or examine", "TASK");
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("d", "dryrun", "dry-run the task, showing debug information");
    opts.optflag("e", "execute", "execute the task");
    opts.optflag("v", "verbose", "use verbose logging");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => {
            err_bad_usage(&format!("{}", f.description()), &program, opts);
            return;
        }
    };

    let process_manifest = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        // Use default manifest
        "manifest.yml".to_string()
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    if matches.opt_present("v") {
        println!("Using debug!");
        pup_enable_debug()
    }

    let mut args: HashMap<PupArg, String> = HashMap::new();
    args.insert(PupArg::ProcessManifestPath, process_manifest);


    // Execute
    if matches.opt_present("t") {
        args.insert(PupArg::TaskId, matches.opt_str("t").unwrap());

        // Dryrun
        if matches.opt_present("d") {
            args.insert(PupArg::DryRun, "1".to_string());
            match pup_main(PupTask::RunTask, args) {
                Ok(_) => process::exit(0),
                Err(err) => {
                    err_failure(err.description());
                    process::exit(1)
                }
            };
        }

        // Actually run
        if matches.opt_present("e") {
            match pup_main(PupTask::RunTask, args) {
                Ok(_) => process::exit(0),
                Err(err) => {
                    err_failure(err.description());
                    process::exit(1)
                }
            };
        }

        // Plan is the default fallback
        match pup_main(PupTask::ShowExecutionPlan, args) {
            Ok(_) => process::exit(0),
            Err(err) => {
                err_failure(err.description());
                process::exit(1)
            }
        };
    }

    // Fallback; list tasks
    args.insert(PupArg::ListTaskVersions, "1".to_string());
    match pup_main(PupTask::ListAvailableTasks, args) {
        Ok(_) => process::exit(0),
        Err(err) => {
            err_failure(err.description());
            process::exit(1)
        }
    };
}
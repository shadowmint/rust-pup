extern crate getopts;
extern crate pup_main;

use std::env;
use getopts::Options;
use pup_main::{pup_enable_debug, pup_main, PupArg, PupTask};
use std::collections::HashMap;
use std::process;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn print_err(error: &str, program: &str, opts: Options) {
    println!("Error: {}", error);
    print_usage(program, opts);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("t", "task", "the id of the task to run or examine", "TASK");
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("a", "all", "list all versions");
    opts.optflag("d", "dryrun", "dry-run execute the task");
    opts.optflag("p", "plan", "show the execution plan for the task");
    opts.optflag("v", "verbose", "use verbose logging");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    let process_manifest = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
        return;
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    if matches.opt_present("v") {
        pup_enable_debug()
    }

    let mut args: HashMap<PupArg, String> = HashMap::new();
    args.insert(PupArg::ProcessManifestPath, process_manifest);

    // Plan
    if matches.opt_present("p") {
        if !matches.opt_present("t") {
            print_err("--plan requires a task id with --task", &program, opts);
            process::exit(1);
        }

        args.insert(PupArg::TaskId, matches.opt_str("t").unwrap());
        match pup_main(PupTask::ShowExecutionPlan, args) {
            Ok(_) => process::exit(0),
            Err(_) => process::exit(1)
        };
    }

    // Dryrun
    if matches.opt_present("d") {
        if !matches.opt_present("t") {
            print_err("--dryrun requires a task id with --task", &program, opts);
            process::exit(1);
        }

        args.insert(PupArg::DryRun, "1".to_string());
        args.insert(PupArg::TaskId, matches.opt_str("t").unwrap());
        match pup_main(PupTask::RunTask, args) {
            Ok(_) => process::exit(0),
            Err(_) => process::exit(1)
        };
    }

    // Execute
    if matches.opt_present("t") {
        args.insert(PupArg::TaskId, matches.opt_str("t").unwrap());
        match pup_main(PupTask::RunTask, args) {
            Ok(_) => process::exit(0),
            Err(_) => process::exit(1)
        };
    }

    // Fallback; list tasks
    if matches.opt_present("a") {
        args.insert(PupArg::ListTaskVersions, "1".to_string());
    }
    match pup_main(PupTask::ListAvailableTasks, args) {
        Ok(_) => process::exit(0),
        Err(_) => process::exit(1)
    };
}
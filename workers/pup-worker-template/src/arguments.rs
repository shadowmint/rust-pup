use pup_worker::errors::PupWorkerError;
use std::env;
use getopts::Options;
use std::process;
use std::error::Error;

const DEFAULT_MANIFEST_PATH: &str = "main.yml";
const DEFAULT_VERBOSE_MODE: bool = false;

pub struct Arguments {
    pub manifest_path: String,
    pub verbose_logging: bool,
}

impl Arguments {
    fn new() -> Arguments {
        return Arguments {
            manifest_path: DEFAULT_MANIFEST_PATH.to_string(),
            verbose_logging: DEFAULT_VERBOSE_MODE,
        };
    }
}

pub fn process_args() -> Result<Arguments, PupWorkerError> {
    let mut data = Arguments::new();
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "show help message");
    opts.optflag("v", "verbose", "use verbose logging");
    opts.optopt("f", "file", "the name of the config file to process", "FILE");

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
        data.verbose_logging = true;
    }

    data.manifest_path = matches.opt_str("f").unwrap_or(DEFAULT_MANIFEST_PATH.to_string());

    return Ok(data);
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn print_err_bad_usage(error: &str, program: &str, opts: Options) {
    println!("Error: {}", error);
    print_usage(program, opts);
}
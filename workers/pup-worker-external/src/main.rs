#[macro_use]
extern crate serde_derive;

extern crate pup_worker;
extern crate serde_yaml;
extern crate handlebars;

use pup_worker::errors::{PupError, PupErrorType};
use pup_worker::utils::path;
use pup_worker::utils::exec;

use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::error::Error;
use std::env;
use handlebars::Handlebars;
use std::collections::HashMap;

fn main() {
    main_();
}

fn main_() {
    let here = env::current_dir().unwrap();
    let mut task = TaskManifest::try_from(&here);
    let mut reg = Handlebars::new();
    let raw = task.args.clone();

    let full_path = find_binary_from_task(&task.task);

    let mut all_vars: HashMap<String, String> = HashMap::new();
    for (key, value) in env::vars() {
        all_vars.insert(key, value);
    }

    for i in 0..raw.len() {
        println!("RAW: {}", raw[i]);
        task.args[i] = reg.render_template(&raw[i], &all_vars).unwrap();
        println!("RENDERED: {}", task.args[i]);
    }

    println!("RUN: {} {}", task.task, task.args.join(" "));
    exec::exec(exec::ExecRequest {
        binary_path: full_path,
        args: task.args.clone(),
        env: all_vars,
    }).unwrap();
}

fn find_binary_from_task(task: &str) -> PathBuf {
    return PathBuf::from(task);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskManifest {
    /// The binary to execute
    pub task: String,

    /// The argument string template (handlebars)
    pub args: Vec<String>,
}

impl TaskManifest {
    pub fn try_from(task_folder: &Path) -> Self {
        let manifest_path = task_folder.join("main.yml");
        println!("{:?}", manifest_path);
        return Self::read_manifest(&manifest_path);
    }

    fn read_manifest(manifest_path: &Path) -> Self {
        let mut fp = File::open(&manifest_path).unwrap();
        let mut raw = String::new();
        fp.read_to_string(&mut raw).unwrap();
        return serde_yaml::from_str(&raw).unwrap();
    }
}

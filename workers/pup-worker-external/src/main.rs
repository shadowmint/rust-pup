#[macro_use]
extern crate serde_derive;

extern crate pup_worker;
extern crate serde_yaml;
extern crate handlebars;

use pup_worker::utils::path;
use pup_worker::utils::exec;

use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::env;
use handlebars::Handlebars;
use std::collections::HashMap;
use std::process;

fn main() {
    let here = env::current_dir().unwrap();
    trace(&format!("folder: {}", path::display(&here)));

    let mut manifest = TaskManifest::try_from(&here);
    let reg = Handlebars::new();

    // Copy environment variable for child
    let mut all_vars: HashMap<String, String> = HashMap::new();
    for (key, value) in env::vars() {
        all_vars.insert(key, value);
    }

    // For each task
    let mut offset = 0;
    let count = manifest.tasks.len();
    for task in manifest.tasks.iter_mut() {
        offset += 1;

        // Debug
        println!();
        if task.info.len() > 0 {
            trace(&format!("running: {} ({}/{})", &task.info, offset, count));
        } else {
            trace(&format!("running: task ({}/{})", offset, count));
        }

        // Find binary to run
        let rendered_exec_path = reg.render_template(&task.task, &all_vars).unwrap();
        let full_path = find_binary_from_task(&rendered_exec_path);

        // Render arguments
        let raw = task.args.clone();
        for i in 0..raw.len() {
            task.args[i] = reg.render_template(&raw[i], &all_vars).unwrap();
        }

        // Move to some other folder if required
        if task.path != "" {
            let rendered_path = reg.render_template(&task.path, &all_vars).unwrap();
            trace(&format!("folder: {}", &rendered_path));
            env::set_current_dir(&rendered_path).unwrap();
        } else {
            trace(&format!("folder: {}", path::display(&here)));
            env::set_current_dir(&here).unwrap();
        }

        // Execute task
        trace(&format!("exec: {} {}", rendered_exec_path, task.args.join(" ")));
        let output = exec::exec(exec::ExecRequest {
            binary_path: full_path,
            args: task.args.clone(),
            env: all_vars.clone(),
            capture: false,
        }).unwrap();

        // Final
        if output.return_code != 0 {
            trace("Subtask failed. Halting.");
            process::exit(1);
        }
    }
}

fn trace(message: &str) {
    println!("pup-worker-external: {}", message);
}

fn find_binary_from_task(task: &str) -> PathBuf {
    return PathBuf::from(task);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskManifest {
    pub tasks: Vec<TaskItem>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskItem {
    /// Some description of the task.
    #[serde(default)]
    pub info: String,

    /// The path to execute in.
    #[serde(default)]
    pub path: String,

    /// The binary to execute
    pub task: String,

    /// The argument string template (handlebars)
    pub args: Vec<String>,
}

impl TaskManifest {
    pub fn try_from(task_folder: &Path) -> Self {
        let manifest_path = task_folder.join("main.yml");
        return Self::read_manifest(&manifest_path);
    }

    fn read_manifest(manifest_path: &Path) -> Self {
        let mut fp = File::open(&manifest_path).unwrap();
        let mut raw = String::new();
        fp.read_to_string(&mut raw).unwrap();
        return serde_yaml::from_str(&raw).unwrap();
    }
}

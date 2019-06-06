#[macro_use]
extern crate serde_derive;

extern crate handlebars;
extern crate pup_worker;
extern crate serde_yaml;
extern crate walkdir;

use pup_worker::utils::exec;
use pup_worker::utils::path;

use handlebars::Handlebars;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use walkdir::WalkDir;

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

    // Get task
    let task = &mut manifest.msbuild;

    // Debug
    println!();
    if task.info.len() > 0 {
        trace(&format!("running: {}", &task.info));
    } else {
        trace("running: task");
    }

    // Find binary to run
    let full_path = find_agent(&task.build_agent, &task.build_agent_version);

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
    trace(&format!(
        "exec: {} {}",
        full_path.to_str().unwrap(),
        task.args.join(" ")
    ));
    let output = exec::exec(exec::ExecRequest {
        binary_path: full_path,
        args: task.args.clone(),
        env: all_vars.clone(),
        capture: false,
    })
    .unwrap();

    // Abort run if it failed
    if output.return_code != 0 {
        trace("Build failed. Halting.");
        process::exit(1);
    }
}

fn trace(message: &str) {
    println!("pup-worker-external: {}", message);
}

fn find_agent(agent: &str, agent_version: &str) -> PathBuf {
    // Find home folder
    let home_dir = dirs::home_dir().unwrap();
    let vswhere_dir = path::join(home_dir, ".vswhere");
    trace(&format!(
        "Looking for vswhere in: {}",
        &vswhere_dir.to_str().unwrap()
    ));

    // Already installed?
    if !path::exists(&vswhere_dir) {
        trace("Missing vswhere. Installing using nuget if we can...");
        trace("exec: nuget install vswhere -o ~/.vswhere");
        exec::exec(exec::ExecRequest {
            binary_path: PathBuf::from("nuget"),
            args: vec!["install", "vswhere", "-o", &vswhere_dir.to_str().unwrap()]
                .into_iter()
                .map(|i| i.to_string())
                .collect(),
            env: HashMap::new(),
            capture: false,
        })
        .unwrap();
    }

    // Get full path to vswhere
    let mut vswhere: Option<PathBuf> = None;
    for entry in WalkDir::new(vswhere_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_file()
            && (entry.path().file_name().unwrap() == "vswhere"
                || entry.path().file_name().unwrap() == "vswhere.exe")
        {
            trace(&format!("Found: {}", entry.path().display()));
            vswhere = Some(PathBuf::from(entry.path()));
            break;
        }
    }
    if vswhere.is_none() {
        trace("Unable to find vswhere. Can't find msbuild.");
        process::exit(1);
    }

    // Setup query args
    let mut version = if agent_version == "latest" {
        vec!["-latest"]
    } else {
        vec!["-version", agent_version]
    };

    // Determine mode
    let agent_path = if agent == "msbuild" {
        // vswhere args
        version.append(&mut vec![
            "-requires",
            "Microsoft.Component.MSBuild",
            "-find",
            "MSBuild\\**\\Bin\\MSBuild.exe",
        ]);
        let args: Vec<String> = version.into_iter().map(|i| i.to_string()).collect();

        // Run vswhere to find msbuild
        trace(&format!("vswhere {}", args.join(" ")));
        let found = exec::exec(exec::ExecRequest {
            binary_path: vswhere.take().unwrap(),
            args,
            env: HashMap::new(),
            capture: true,
        })
        .unwrap();

        // Take first result
        let cleaned = found.stdout.unwrap().to_string();
        let mut lines = cleaned.lines().into_iter().map(|i| i.trim().to_string());
        let fullpath = lines.next().unwrap_or("NOT_FOUND".to_string());
        trace(&format!("vswhere returned: {}", cleaned));
        fullpath
    } else if agent == "devenv" {
        // vswhere args
        version.append(&mut vec!["-find", "**\\devenv.exe"]);
        let args: Vec<String> = version.into_iter().map(|i| i.to_string()).collect();

        // Run vswhere to find msbuild
        trace(&format!("vswhere {}", args.join(" ")));
        let found = exec::exec(exec::ExecRequest {
            binary_path: vswhere.take().unwrap(),
            args,
            env: HashMap::new(),
            capture: true,
        })
        .unwrap();

        // Take first result
        let cleaned = found.stdout.unwrap().to_string();
        let mut lines = cleaned.lines().into_iter().map(|i| i.trim().to_string());
        let fullpath = lines.next().unwrap_or("NOT_FOUND".to_string());
        trace(&format!("vswhere returned: {}", cleaned));
        fullpath
    } else {
        trace(&format!(
            "Invalid agent: {}. Try 'msbuild' or 'devenv'",
            agent
        ));
        process::exit(1);
    };

    if !path::exists(&agent_path) {
        trace("vswhere didn't return a valid path. Can't find agent.");
        process::exit(1);
    }

    trace(&format!("Found msbuild: {}", path::display(&agent_path)));
    return PathBuf::from(agent_path);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskManifest {
    pub msbuild: TaskItem,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskItem {
    /// Some description of the task.
    #[serde(default)]
    pub info: String,

    /// Pick a specific build version
    #[serde(default = "default_agent_version")]
    #[serde(rename = "buildAgentVersion")]
    pub build_agent_version: String,

    /// Use devenv instead of msbuild
    #[serde(default = "default_build_agent")]
    #[serde(rename = "buildAgent")]
    pub build_agent: String,

    /// The path to execute in.
    #[serde(default)]
    pub path: String,

    /// The argument string template (handlebars)
    pub args: Vec<String>,
}

fn default_build_agent() -> String {
    format!("msbuild")
}

fn default_agent_version() -> String {
    format!("latest")
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

#[macro_use]
extern crate serde_derive;

extern crate base_logging;
extern crate handlebars;
extern crate pup_worker;
extern crate serde_yaml;

mod errors;
mod internal_exec;
mod manifest;

use crate::internal_exec::exec_detached;
use crate::manifest::{TaskItem, TaskManifest};

use base_logging::{Level, Logger};
use pup_worker::logger::get_logger;
use pup_worker::utils::exec;
use pup_worker::utils::path;
use std::ops::DerefMut;

use crate::errors::WorkerError;
use handlebars::Handlebars;
use pup_worker::errors::{PupWorkerError, PupWorkerErrorType};
use pup_worker::utils::exec::ExecResult;
use std::collections::HashMap;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use std::sync::Mutex;

fn main() -> Result<(), PupWorkerError> {
    ExternalTask::new()?.execute()
}

struct ExternalTask {
    logger: Logger,
}

impl ExternalTask {
    pub fn new() -> Result<ExternalTask, PupWorkerError> {
        Ok(ExternalTask {
            logger: get_logger()?,
        })
    }

    fn execute(&mut self) -> Result<(), PupWorkerError> {
        let here = env::current_dir().unwrap();
        self.trace(&format!("folder: {}", path::display(&here)));

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
            match self.execute_single_task(task, offset, count, &reg, &all_vars, &here) {
                Err(e) => {
                    if task.continue_on_failure {
                        self.trace(&format!(
                            "Subtask failed: {:?}, but continueOnFailure set.",
                            e
                        ));
                    } else {
                        self.trace(&format!("Subtask failed. Halting.\n{:?}", e));
                        process::exit(1);
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn execute_single_task(
        &mut self,
        task: &mut TaskItem,
        offset: i32,
        count: usize,
        reg: &Handlebars,
        all_vars: &HashMap<String, String>,
        here: &PathBuf,
    ) -> Result<(), WorkerError> {
        // Debug
        println!();
        if !task.info.is_empty() {
            self.trace(&format!("running: {} ({}/{})", &task.info, offset, count));
        } else {
            self.trace(&format!("running: task ({}/{})", offset, count));
        }

        // Process task path
        let mut paths = Vec::new();
        let rendered_exec_path = reg.render_template(&task.task, &all_vars)?;
        let full_path = self.find_binary_from_task(&rendered_exec_path);
        paths.push(full_path);

        // Process variant paths
        for variant_path in task.task_variants.iter() {
            let rendered_exec_path = reg.render_template(variant_path.as_str(), &all_vars)?;
            let full_path = self.find_binary_from_task(&rendered_exec_path);
            paths.push(full_path);
        }

        // Find output path
        let rendered_output_path = reg.render_template(&task.output, &all_vars)?;
        let full_output_path = PathBuf::from(&rendered_output_path);

        // Render arguments
        let raw = task.args.clone();
        for i in 0..raw.len() {
            task.args[i] = reg.render_template(&raw[i], &all_vars)?;
        }

        // Move to some other folder if required
        if task.path != "" {
            let rendered_path = reg.render_template(&task.path, &all_vars)?;
            self.trace(&format!("folder: {}", &rendered_path));
            env::set_current_dir(&rendered_path)?;
        } else {
            self.trace(&format!("folder: {}", path::display(&here)));
            env::set_current_dir(&here)?;
        }

        // Execute a detached task; special hack for stupid long lived processes
        // or... execute a normal task and wait for the response.

        let output = self.try_exec_variants(
            paths,
            task,
            all_vars,
            rendered_output_path,
            full_output_path,
        )?;

        // Final check on result status
        if output.return_code != 0 {
            return Err(WorkerError::FailureReturnCode);
        }

        Ok(())
    }

    fn try_exec_variants(
        &mut self,
        paths: Vec<PathBuf>,
        task: &mut TaskItem,
        all_vars: &HashMap<String, String>,
        rendered_output_path: String,
        full_output_path: PathBuf,
    ) -> Result<ExecResult, PupWorkerError> {
        for potential_cmd in paths.iter() {
            match self.try_exec_variant(
                potential_cmd.clone(),
                task,
                all_vars,
                rendered_output_path.clone(),
                full_output_path.clone(),
            ) {
                Ok(result) => {
                    return Ok(result);
                }
                Err(_) => {
                    // Try another variant
                }
            }
        }
        Err(PupWorkerError::with_message(
            PupWorkerErrorType::InvalidRequest,
            "Unable to execute command",
        ))
    }

    fn try_exec_variant(
        &mut self,
        full_path: PathBuf,
        task: &mut TaskItem,
        all_vars: &HashMap<String, String>,
        rendered_output_path: String,
        full_output_path: PathBuf,
    ) -> Result<ExecResult, PupWorkerError> {
        self.trace(&format!(
            "exec: {} {}",
            full_path.to_string_lossy(),
            task.args.join(" ")
        ));
        let output = if task.dont_wait {
            exec_detached(exec::ExecRequest {
                binary_path: full_path,
                args: task.args.clone(),
                env: all_vars.clone(),
                capture: false,
            })?
        } else if task.output != "" {
            self.trace(&format!("output: {}", rendered_output_path));
            let fp = match OpenOptions::new()
                .write(true)
                .create(true)
                .open(&full_output_path)
            {
                Ok(v) => v,
                Err(err) => {
                    return Err(PupWorkerError::with_message(
                        PupWorkerErrorType::IOError,
                        &format!("{}", err),
                    ));
                }
            };

            let stdout_out = Arc::new(Mutex::new(fp));
            let stderr_out = stdout_out.clone();
            internal_exec::exec_stream(
                exec::ExecRequest {
                    binary_path: full_path,
                    args: task.args.clone(),
                    env: all_vars.clone(),
                    capture: false,
                },
                move |line: &str| {
                    println!("{}", line);
                    writeln!(stdout_out.lock().unwrap().deref_mut(), "{}", line).unwrap();
                },
                move |line: &str| {
                    println!("{}", line);
                    writeln!(stderr_out.lock().unwrap().deref_mut(), "{}", line).unwrap();
                },
            )?
        } else {
            exec::exec(exec::ExecRequest {
                binary_path: full_path,
                args: task.args.clone(),
                env: all_vars.clone(),
                capture: false,
            })?
        };
        Ok(output)
    }

    fn trace(&mut self, message: &str) {
        self.logger
            .log(Level::Info, format!("pup-worker-external: {}", message));
    }

    fn find_binary_from_task(&mut self, task: &str) -> PathBuf {
        PathBuf::from(task)
    }
}

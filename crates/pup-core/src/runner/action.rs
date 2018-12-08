use crate::context::PupContext;
use crate::errors::{PupError, PupErrorType};
use crate::task::PupTask;
use crate::manifest::PupManifestVersion;
use crate::worker::{PupWorker, PupWorkerResult};
use crate::base_logging::Level;
use crate::logger::get_logger;
use crate::dunce;
use crate::time;
use base_logging::Logger;
use std::env;
use std::path::Path;
use std::thread::{spawn, JoinHandle};
use std::error::Error;
use crate::runner::ExecRequest;
use crate::runner::exec::exec;
use std::path::PathBuf;
use crate::runner::ExecResult;
use std::collections::HashMap;
use crate::utils::path;
use time::Duration;
use time::Tm;
use crate::runner::env::EnvHelper;
use crate::manifest::PupManifestStep;

/// An action that involves executing an external command
#[derive(Clone)]
pub struct PupExternalAction {
    /// The task to use
    pub task: PupTask,

    /// The version of the task to use
    pub version: PupManifestVersion,

    /// The external runner
    pub worker: PupWorker,

    /// The result bucker for the external runner
    pub result: PupWorkerResult,

    /// The env to use for this specific instance
    pub env: HashMap<String, String>,
}

/// A task to be execute by the runner
#[derive(Clone)]
pub struct PupAction {
    /// Did this complete?
    pub completed: bool,

    /// Was this a success?
    pub success: bool,

    /// Details for an external
    pub external: Option<PupExternalAction>,

    /// Any child actions
    pub children: Vec<PupAction>,
}

/// Options to use when 
#[derive(Clone)]
pub struct PupActionOptions {
    /// Is this a dry run? If so, don't actually execute the task.
    /// Is this a dry run? If so, don't actually execute the task.
    pub dry_run: bool,

    /// The set of distinct arguments to pass to the final executable.
    pub args: Vec<String>,
}

impl PupAction {
    /// Return a new blank action
    pub fn new() -> PupAction {
        return PupAction {
            children: Vec::new(),
            success: false,
            completed: false,
            external: None,
        };
    }

    /// Attempt to load the task and all children from the given context
    pub fn load(&mut self, context: &PupContext, name: &str, parent_env: &HashMap<String, String>) -> Result<(), PupError> {
        // TODO: Recursive runaway check here

        // Load task and version
        let mut logger = get_logger();
        logger.log(Level::Debug, format!("Loading task: {}", name));
        let maybe_task = context.load_task(name);
        if maybe_task.is_err() {
            logger.log(Level::Debug, format!("Failed to load task: {}: {}", name, maybe_task.as_ref().err().unwrap().description()));
            return Err(maybe_task.err().unwrap());
        }
        let (task, version) = maybe_task.unwrap();

        // Load the worker for the task
        let maybe_worker = context.load_worker(&task.manifest.action);
        if maybe_worker.is_err() {
            logger.log(Level::Debug, format!("Failed to load worker: {}: {}", name, maybe_worker.as_ref().err().unwrap().description()));
            return Err(maybe_worker.err().unwrap());
        }
        let worker = maybe_worker.unwrap();

        // Load children
        let env_helper = EnvHelper::new();
        for step in version.steps.iter() {

            // Generate a combined env for this child
            let env = match env_helper.extend_with_parent_env(&step.environment, parent_env) {
                Ok(e) => e,
                Err(err) => {
                    logger.log(Level::Debug, format!("Failed to load task: {}: {}", name, err.description()));
                    return Err(err);
                }
            };

            if self.should_skip_task(&env_helper, step, &env, name, &mut logger)? {
                continue;
            }

            // Load the child with the rendered env group
            logger.log(Level::Debug, format!("Loading child task: {}", step.step));
            let mut child_action = PupAction::new();
            child_action.load(context, &step.step, &env)?;
            self.children.push(child_action);
        }

        // Configure self
        self.external = Some(PupExternalAction {
            worker,
            task,
            version,
            result: PupWorkerResult {},
            env: parent_env.clone(),
        });

        Ok(())
    }

    /// 'skip' and 'if' are two special markers on steps to decide if they should execute in a plan.
    fn should_skip_task(&self, env_helper: &EnvHelper, step: &PupManifestStep, env: &HashMap<String, String>, name: &str, logger: &mut Logger) -> Result<bool, PupError> {
        // Check if this child has a skip marker?
        if step.skip != "" {
            let skip_test = match env_helper.process_env_variable(&step.skip, &env) {
                Ok(v) => v,
                Err(err) => {
                    logger.log(Level::Debug, format!("Failed to load task: {}: {}", name, err.description()));
                    return Err(err);
                }
            };
            if skip_test.len() > 0 && skip_test != "0" && skip_test.to_lowercase() != "false" {
                logger.log(Level::Debug, format!("Skipped child task: {}: skip token was: {}", step.step, skip_test));
                return Ok(true);
            } else {
                if skip_test.len() > 0 {
                    logger.log(Level::Debug, format!("Using optional child task: {}: skip token was: {}", step.step, skip_test));
                }
            }
        }

        // Check if this child has an if marker?
        if step.if_marker != "" {
            let skip_test = match env_helper.process_env_variable(&step.if_marker, &env) {
                Ok(v) => v,
                Err(err) => {
                    logger.log(Level::Debug, format!("Failed to load task: {}: {}", name, err.description()));
                    return Err(err);
                }
            };
            if skip_test.len() == 0 || skip_test == "0" || skip_test.to_lowercase() == "false" {
                logger.log(Level::Debug, format!("Skipped child task: {}: if token was: {}", step.step, skip_test));
                return Ok(true);
            } else {
                if skip_test.len() > 0 {
                    logger.log(Level::Debug, format!("Using optional child task: {}: if token was: {}", step.step, skip_test));
                }
            }
        }

        return Ok(false);
    }

    /// Run this task and all child tasks
    pub fn run(&mut self, logger: &mut Logger, options: &PupActionOptions) -> Result<(), PupError> {
        if options.dry_run {
            self.info(logger, "Dryrun. No tasks will be executed", 1);
        }
        return self.run_timed(logger, options, 1);
    }

    /// Run this task and all child tasks, timed
    fn run_timed(&mut self, logger: &mut Logger, options: &PupActionOptions, depth: usize) -> Result<(), PupError> {
        let time_start = time::now();
        let rtn = self.run_internal(logger, options, depth, time_start);
        let time_stop = time::now();
        let task_duration = time_stop - time_start;

        let result = match rtn.is_err() {
            true => "FAILED",
            false => "Finished"
        };

        match self.external {
            Some(ref ext) => {
                self.info(logger, &format!(
                    "{} task: {} #{} ({}, {})",
                    result,
                    ext.task.name,
                    ext.version.version,
                    format_time(time_stop),
                    format_duration(task_duration)), depth + 1);
            }
            None => {
                self.info(logger, &format!(
                    "{} task ({}, {})",
                    result,
                    format_time(time_stop),
                    format_duration(task_duration)), depth + 1);
            }
        }

        return rtn;
    }

    /// Run this task and all child tasks
    fn run_internal(&mut self, logger: &mut Logger, options: &PupActionOptions, depth: usize, time_start: Tm) -> Result<(), PupError> {
        match self.external {
            Some(ref ext) => {
                self.info(logger, &format!("Entering task: {} #{} ({})", ext.task.name, ext.version.version, format_time(time_start)), depth);
            }
            None => {
                self.info(logger, &format!("Entering task ({})", format_time(time_start)), depth);
            }
        }

        // Execute dependency steps first
        for child in self.children.iter_mut() {
            child.run_timed(logger, options, depth + 1)?;
        }

        // Now execute our own step, if required.
        match self.external {
            Some(ref ext) => {
                // Move to folder
                self.info(logger, &format!("Using: {}", path::display(&ext.version.path)), depth + 1);
                self.try_run_cwd(&ext.version.path, logger, depth)?;

                // Invoke worker stream
                if options.dry_run {
                    self.info(logger, &format!("Exec: (skipped) {} {}", path::display(&ext.worker.path), options.args.join(" ")), depth + 1);
                    let mut keys: Vec<String> = ext.env.keys().map(|i| i.to_string()).collect();
                    keys.sort();
                    for key in keys.iter() {
                        self.info(logger, &format!("Env: {}: {}", key, ext.env[key]), depth + 2);
                    }
                } else {
                    self.info(logger, &format!("Exec: {} {}", path::display(&ext.worker.path), options.args.join(" ")), depth + 1);
                    match try_run_task(&ext.worker.path, options, &ext.env).join() {
                        Ok(result) => {
                            match result {
                                Ok(exec_result) => {
                                    if exec_result.return_code != 0 {
                                        return Err(PupError::with_message(
                                            PupErrorType::WorkerFailed,
                                            &format!("Worker returned exit code: {}", exec_result.return_code),
                                        ));
                                    }
                                    return Ok(());
                                }
                                Err(err) => {
                                    return Err(PupError::with_message(
                                        PupErrorType::WorkerFailed,
                                        &format!("Failed to execute worker: {:?}", err),
                                    ));
                                }
                            }
                        }
                        Err(err) => {
                            return Err(PupError::with_message(
                                PupErrorType::WorkerFailed,
                                &format!("Failed to execute worker: {:?}", err),
                            ));
                        }
                    };
                }
            }
            None => {}
        }

        return Ok(());
    }

    fn info(&self, logger: &mut Logger, message: &str, depth: usize) {
        let prefix = "--".repeat(depth);
        logger.log(Level::Info, format!("{} {}", prefix, message));
    }

    fn try_run_cwd(&self, path: &Path, logger: &mut Logger, depth: usize) -> Result<(), PupError> {
        // Convert UNC paths on windows, because it breaks things.
        // Seriously. Powershell for example won't run without a signed certificate.
        let path_to_use = match dunce::canonicalize(path) {
            Ok(p) => p,
            Err(err) => {
                self.info(logger, &format!("Error converting path: {}", err.description()), depth + 1);
                PathBuf::from(path)
            }
        };

        return match env::set_current_dir(path_to_use) {
            Ok(_) => {
                Ok(())
            }
            Err(err) => {
                Err(PupError::with_error(
                    PupErrorType::MissingWorkFolder,
                    &format!("Failed to set current dir to: {:?}", path),
                    err,
                ))
            }
        };
    }
}

fn try_run_task(binary_path: &Path, options: &PupActionOptions, env: &HashMap<String, String>) -> JoinHandle<Result<ExecResult, PupError>> {
    let owned_path = PathBuf::from(binary_path);
    let owned_options = options.clone();
    let owned_env = env.clone();
    return spawn(move || {
        return exec(ExecRequest {
            env: owned_env,
            binary_path: owned_path,
            args: owned_options.args,
        });
    });
}

fn format_duration(d: Duration) -> String {
    let mut seconds = d.num_seconds();
    let minutes: i64 = seconds / 60;
    if minutes > 0 {
        seconds = seconds - minutes * 60;
    }
    let ms = d.num_milliseconds() - seconds * 1000;
    return match minutes {
        m if m > 0 => format!("{}min {}.{:04}s", m, seconds, ms),
        _ => format!("{}.{:04}s", seconds, ms)
    };
}

fn format_time(tm: Tm) -> String {
    let timestring = match time::strftime("%b %d %H:%M:%S", &tm) {
        Ok(i) => i,
        Err(_) => String::from("(unknown time)")
    };
    return timestring;
}
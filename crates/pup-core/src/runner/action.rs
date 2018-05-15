use ::context::PupContext;
use ::errors::{PupError, PupErrorType};
use ::task::PupTask;
use ::manifest::PupManifestVersion;
use ::worker::{PupWorker, PupWorkerResult};
use ::base_logging::Level;
use ::logger::get_logger;
use ::dunce;
use base_logging::Logger;
use std::env;
use std::path::Path;
use std::thread::{spawn, JoinHandle};
use std::error::Error;
use runner::ExecRequest;
use runner::exec::exec;
use std::path::PathBuf;
use runner::ExecResult;
use std::collections::HashMap;
use utils::path;

/// An action that involves executing an external command
pub struct PupExternalAction {
    /// The task to use
    pub task: PupTask,

    /// The version of the task to use
    pub version: PupManifestVersion,

    /// The external runner
    pub worker: PupWorker,

    /// The result bucker for the external runner
    pub result: PupWorkerResult,
}

/// A task to be execute by the runner
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
    pub fn load(&mut self, context: &PupContext, name: &str) -> Result<(), PupError> {
        // TODO: Recursive runaway check here
        // Load self
        let mut logger = get_logger();
        logger.log(Level::Debug, format!("Loading task: {}", name));

        let maybe_task = context.load_task(name);
        if maybe_task.is_err() {
            logger.log(Level::Warn, format!("Failed to load task: {}: {}", name, maybe_task.as_ref().err().unwrap().description()));
            return Err(maybe_task.err().unwrap());
        }
        let (task, version) = maybe_task.unwrap();

        let maybe_worker = context.load_worker(&task.manifest.action);
        if maybe_worker.is_err() {
            logger.log(Level::Warn, format!("Failed to load worker: {}: {}", name, maybe_worker.as_ref().err().unwrap().description()));
            return Err(maybe_worker.err().unwrap());
        }
        let worker = maybe_worker.unwrap();

        // Load children
        for child_ident in &version.steps {
            logger.log(Level::Debug, format!("Loading child task: {}", child_ident));
            let mut child_action = PupAction::new();
            child_action.load(context, &child_ident)?;
            self.children.push(child_action);
        }

        // Configure self
        self.external = Some(PupExternalAction {
            worker,
            task,
            version,
            result: PupWorkerResult {},
        });

        Ok(())
    }

    /// Run this task and all child tasks
    pub fn run(&mut self, logger: &mut Logger, options: &PupActionOptions) -> Result<(), PupError> {
        if options.dry_run {
            self.info(logger, "Dryrun. No tasks will be executed", 1);
        }
        self.run_internal(logger, options, 1)?;
        return Ok(());
    }

    /// Run this task and all child tasks
    fn run_internal(&mut self, logger: &mut Logger, options: &PupActionOptions, depth: usize) -> Result<(), PupError> {
        match self.external {
            Some(ref ext) => {
                self.info(logger, &format!("Entering task: {} #{}", ext.task.name, ext.version.version), depth);
            }
            None => {
                self.info(logger, "Running tasks", depth);
            }
        }
        // Execute dependency steps first
        for child in self.children.iter_mut() {
            child.run_internal(logger, options, depth + 1)?;
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
                } else {
                    self.info(logger, &format!("Exec: {} {}", path::display(&ext.worker.path), options.args.join(" ")), depth + 1);
                    match try_run_task(&ext.worker.path, options, &ext.worker.env).join() {
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

                self.info(logger, &format!("Finished task: {} #{}", ext.task.name, ext.version.version), depth + 1);
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
                    PupErrorType::MissingVersionFolder,
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
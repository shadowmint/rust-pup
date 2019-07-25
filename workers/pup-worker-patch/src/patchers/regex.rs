use crate::patch::Patcher;
use std::path::PathBuf;
use crate::manifest::PatchTask;
use pup_worker::errors::{PupWorkerError, PupWorkerErrorType};
use std::fs::File;
use pup_worker::logger::get_logger;
use std::io::BufReader;
use pup_worker::logger::Level;
use std::io::BufRead;
use std::io::Write;
use pup_worker::utils::path;
use regex::Regex;
use regex::Replacer;
use regex::Captures;

pub struct RegexPatcher {
    patterns: Vec<RegexPatch>
}

struct RegexPatch {
    regex: Regex,
    values: Vec<String>,
    allow_partial_matching: bool,
}

impl RegexPatcher {
    pub fn new() -> RegexPatcher {
        return RegexPatcher {
            patterns: Vec::new()
        };
    }

    /// Prepare tasks by parsing internal data
    fn prep(&mut self, task: &PatchTask) -> Result<(), PupWorkerError> {
        for step in task.patch.iter() {
            let pattern = RegexPatch {
                regex: PupWorkerError::wrap(Regex::new(&step.pattern))?,
                values: step.values.clone(),
                allow_partial_matching: step.partial,
            };
            self.patterns.push(pattern);
        }
        Ok(())
    }

    /// Patch a single line
    fn patch_line(&self, input: &str) -> (String, bool) {
        let mut output = input.to_string();
        let mut patched = false;
        for pattern in self.patterns.iter() {
            let mut patch = RegexPatchReplacer { step: pattern, patched: false };
            output = pattern.regex.replace(&output, &mut patch).to_string();
            patched |= patch.patched;
        }
        return (output, patched);
    }
}

impl Patcher for RegexPatcher {
    fn patch(&mut self, input: PathBuf, output: PathBuf, task: &PatchTask) -> Result<(), PupWorkerError> {
        let mut logger = get_logger()?;
        self.prep(task)?;

        let input_fp = PupWorkerError::wrap(File::open(&input))?;
        let mut output_fp = PupWorkerError::wrap(File::create(&output))?;

        let input_reader = BufReader::new(input_fp);
        for line in input_reader.lines() {
            match line {
                Ok(l) => {
                    let (mut out, changed) = self.patch_line(&l);
                    if changed {
                        logger.log(Level::Info, format!(" Patch: {}", &l));
                        logger.log(Level::Info, format!("      : {}", &out));
                    }
                    out.push_str("\n"); // Always add a newline to the file
                    PupWorkerError::wrap(output_fp.write(out.as_bytes()))?;
                }
                Err(err) => {
                    return Err(PupWorkerError::with_error(
                        PupWorkerErrorType::IOError,
                        &format!("Failed while reading file: {}", path::display(&input)),
                        err)
                    );
                }
            }
        }

        Ok(())
    }
}

struct RegexPatchReplacer<'a> {
    step: &'a RegexPatch,
    patched: bool,
}

impl<'a, 'b> Replacer for &'a mut RegexPatchReplacer<'b> {
    fn replace_append(&mut self, caps: &Captures, dst: &mut String) {
        // NB. The -1 is because the first match is always the full string match.
        let actual_matches = caps.iter().filter(|i| i.is_some()).count() - 1;

        // Everything must be relative to the start offset
        let full = &caps[0];
        let full_origin = caps.get(0).unwrap().start();

        if actual_matches > 0 && (self.step.allow_partial_matching || actual_matches == self.step.values.len()) {
            let mut last_read = 0;
            let mut offset = 0;
            for cap in caps.iter().skip(1) {
                match cap {
                    Some(c) => {
                        let len = (c.end() - full_origin) - (c.start() - full_origin);
                        if len > 0 {
                            // Do we need to add a prefix?
                            let prefix_len = (c.start() - full_origin) - last_read;
                            if prefix_len > 0 {
                                full.chars().skip(last_read).take(prefix_len).map(|i| dst.push(i)).count();
                            }
                            dst.push_str(&self.step.values[offset]);
                            offset += 1;
                            last_read = c.end() - full_origin;
                        }
                    }
                    None => {
                        // Skip the replace value
                        offset += 1;
                    }
                }
            }

            // If there are left over values, apply them.
            let postfix_len = full.len() - last_read;
            if postfix_len > 0 {
                full.chars().skip(last_read).take(postfix_len).map(|i| dst.push(i)).count();
            }

            self.patched = true;
        } else {
            dst.push_str(full);
        }
    }
}
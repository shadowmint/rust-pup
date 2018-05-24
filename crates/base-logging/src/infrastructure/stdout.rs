use std::sync::{Once, ONCE_INIT};
use std::io::stdout as stdout_factory;
use std::io::stderr as stderr_factory;
use std::io::Stdout;
use std::io::Stderr;
use std::io::Write;
use std::io::StdoutLock;
use std::io::StderrLock;
use std::mem::transmute;

static STDOUT_INIT: Once = ONCE_INIT;
static mut STDOUT: *mut Stdout = 0 as *mut Stdout;

static STDERR_INIT: Once = ONCE_INIT;
static mut STDERR: *mut Stderr = 0 as *mut Stderr;

#[allow(dead_code)]
pub fn stderr<'a>() -> StderrLock<'a> {
    STDERR_INIT.call_once(|| {
        unsafe {
            STDERR = transmute(Box::new(stderr_factory()));
        }
    });
    unsafe {
        return (*STDERR).lock();
    }
}

#[allow(dead_code)]
fn stdout<'a>() -> StdoutLock<'a> {
    STDOUT_INIT.call_once(|| {
        unsafe {
            STDOUT = transmute(Box::new(stdout_factory()));
        }
    });
    unsafe {
        return (*STDOUT).lock();
    }
}

/// Debug print immediately to stdout
pub fn debug(message: &str) {
    let mut guard = stdout();
    let _ = guard.write(message.as_bytes());
    let _ = guard.write(&['\n' as u8]);
}
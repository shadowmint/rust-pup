use utils::path::join;
use std::path::PathBuf;
use std::collections::HashMap;
use std::env::current_exe;
use PupProcess;

pub fn test_fixture() -> PupProcess {
    let root = test_context_process_path();

    // Fake some external environment variables
    let mut overrides: HashMap<String, String> = HashMap::new();
    overrides.insert("EXT_USERNAME".to_string(), "foouser".to_string());
    overrides.insert("EXT_PASSWORD".to_string(), "foopass".to_string());

    let process = PupProcess::load_from(root, Some(overrides)).unwrap();
    return process;
}

pub fn test_context_process_path() -> PathBuf {
    return join(&test_context_folder(), "dev.yml");
}

pub fn test_context_folder() -> PathBuf {
    let test_exe = PathBuf::from(current_exe().unwrap());
    let test_exe_folder = test_exe.parent().unwrap();
    let test_data_folder = test_exe_folder.join("..").join("..").join("..").join("..").join("..").join("sample");
    return test_data_folder.canonicalize().unwrap();
}
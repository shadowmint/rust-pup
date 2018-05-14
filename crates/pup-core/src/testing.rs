use ::context::PupContext;
use utils::path::join;
use std::path::PathBuf;
use std::collections::HashMap;
use std::env::current_exe;

pub fn test_context_fixture() -> PupContext {
    let root = test_context_folder();
    let mut context = PupContext::new(
        &join(&root, "tasks"),
        &join(&root, "workers"));

    let mut fake_env: HashMap<String, String> = HashMap::new();
    fake_env.insert(String::from("foo"), String::from("bar"));
    fake_env.insert(String::from("config"), String::from("test"));

    context.set_environment(&fake_env);
    return context;
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
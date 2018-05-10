use ::context::PupContext;
use std::env::current_dir;
use utils::path::join;
use std::path::PathBuf;

pub fn test_context_fixture() -> PupContext {
    let core_folder = PathBuf::from(current_dir().unwrap());
    let root = core_folder.join("data");

    let context = PupContext::new(
        &join(&root, "config/debug.json"),
        &join(&root, "tasks"),
        &join(&root, "workers"));

    return context;
}

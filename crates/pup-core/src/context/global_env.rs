use crate::utils::path::display;
use std::collections::HashMap;
use std::path::Path;

/// Generate a global injected env to use for everything
pub fn build_global_env(root: &Path) -> HashMap<String, String> {
    let mut global_env = HashMap::new();
    global_env.insert(format!("MANIFEST_HOME"), display(root));
    return global_env;
}

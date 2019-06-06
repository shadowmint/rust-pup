use std::collections::HashMap;
use handlebars::Handlebars;
use std::env;
use errors::PupWorkerError;

/// Renderer to render env variables before passing to workers.
pub struct EnvHelper {
    ambient: Option<HashMap<String, String>>,
    renderer: Handlebars
}

impl EnvHelper {
    pub fn new() -> EnvHelper {
        return EnvHelper {
            ambient: None,
            renderer: Handlebars::new(),
        };
    }

    pub fn expand_single_value(&self, source: &str, parent_env: &HashMap<String, String>) -> Result<String, PupWorkerError> {
        let output = PupWorkerError::wrap(self.renderer.render_template(source, parent_env))?;
        return Ok(output);
    }

    pub fn extend_with_parent_env(&self, source: &HashMap<String, String>, parent_env: &HashMap<String, String>) -> Result<HashMap<String, String>, PupWorkerError> {
        // Prepopulate with parent
        let mut rtn = parent_env.clone();

        // Render each child key, using the parent array
        for key in source.keys() {
            let new_value = self.expand_single_value(&source[key], parent_env)?;
            rtn.insert(key.to_string(), new_value);
        }

        return Ok(rtn);
    }

    pub fn render_existing_keys_from_parent_scope(&self, source: &HashMap<String, String>, parent_env: &HashMap<String, String>) -> Result<HashMap<String, String>, PupWorkerError> {
        // Prepopulate with parent
        let mut rtn = source.clone();

        // Render each child key, using the parent array
        for key in source.keys() {
            let new_value = self.expand_single_value(&source[key], parent_env)?;
            rtn.insert(key.to_string(), new_value);
        }

        return Ok(rtn);
    }

    pub fn ambient_state(&mut self) -> &HashMap<String, String> {
        if self.ambient.is_some() {
            return &self.ambient.as_ref().unwrap();
        }

        let mut rtn: HashMap<String, String> = HashMap::new();
        for (key, value) in env::vars() {
            rtn.insert(key, value);
        }
        self.ambient = Some(rtn);
        return &self.ambient.as_ref().unwrap();
    }
}
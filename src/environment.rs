use std::collections::HashMap;
use std::path::Path;
use std::fs;

use crate::parser::{parse_environment, ast::{Environment, BruFile}};

pub fn load_environment(bru_file_path: &Path, env_name: &str) -> Result<Environment, String> {
    let parent = bru_file_path.parent().ok_or("Cannot get parent directory")?;
    let env_path = parent.join("environments").join(format!("{}.bru", env_name));

    if !env_path.exists() {
        return Err(format!("Environment file not found: {:?}", env_path));
    }

    let content = fs::read_to_string(&env_path)
        .map_err(|e| format!("Cannot read environment file: {}", e))?;

    parse_environment(&content)
}

pub fn apply_environment(bru: &mut BruFile, env: &Environment) {
    bru.request.url = substitute_variables(&bru.request.url, &env.vars);

    if let Some(ref mut body) = bru.body {
        body.content = substitute_variables(&body.content, &env.vars);
    }

    for value in bru.headers.values_mut() {
        *value = substitute_variables(value, &env.vars);
    }
}

fn substitute_variables(text: &str, vars: &HashMap<String, String>) -> String {
    let mut result = text.to_string();

    for (key, value) in vars {
        let pattern = format!("{{{{{}}}}}", key);
        result = result.replace(&pattern, value);
    }

    result
}

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

use crate::parser::{parse_environment, ast::{Environment, BruFile}};

fn find_collection_root(bru_file_path: &Path) -> Result<PathBuf, String> {
    let mut current = bru_file_path.parent();

    while let Some(dir) = current {
        if dir.join("bruno.json").exists() {
            return Ok(dir.to_path_buf());
        }
        current = dir.parent();
    }

    bru_file_path
        .parent()
        .map(PathBuf::from)
        .ok_or_else(|| "Cannot determine collection root".to_string())
}

pub fn load_environment(bru_file_path: &Path, env_name: &str) -> Result<Environment, String> {
    let collection_root = find_collection_root(bru_file_path)?;
    let env_path = collection_root.join("environments").join(format!("{}.bru", env_name));

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::{Body, Method, Request};
    use tempfile::TempDir;

    fn create_bruno_collection(temp: &TempDir, nested_path: &str) -> PathBuf {
        let root = temp.path();
        fs::write(root.join("bruno.json"), "{}").unwrap();

        let env_dir = root.join("environments");
        fs::create_dir_all(&env_dir).unwrap();
        fs::write(
            env_dir.join("LOCAL.bru"),
            "vars {\n  BASE_URL: http://localhost:3000\n}\n",
        ).unwrap();

        let bru_dir = root.join(nested_path);
        fs::create_dir_all(&bru_dir).unwrap();

        let bru_file = bru_dir.join("request.bru");
        fs::write(&bru_file, "get {\n  url: {{BASE_URL}}/api\n}\n").unwrap();

        bru_file
    }

    #[test]
    fn find_collection_root_from_nested_dir() {
        let temp = TempDir::new().unwrap();
        let bru_file = create_bruno_collection(&temp, "folder/subfolder/requests");

        let root = find_collection_root(&bru_file).unwrap();
        assert_eq!(root, temp.path());
    }

    #[test]
    fn find_collection_root_from_direct_child() {
        let temp = TempDir::new().unwrap();
        let bru_file = create_bruno_collection(&temp, "requests");

        let root = find_collection_root(&bru_file).unwrap();
        assert_eq!(root, temp.path());
    }

    #[test]
    fn find_collection_root_fallback_when_no_bruno_json() {
        let temp = TempDir::new().unwrap();
        let bru_dir = temp.path().join("requests");
        fs::create_dir_all(&bru_dir).unwrap();

        let bru_file = bru_dir.join("request.bru");
        fs::write(&bru_file, "get { url: http://example.com }").unwrap();

        let root = find_collection_root(&bru_file).unwrap();
        assert_eq!(root, bru_dir);
    }

    #[test]
    fn load_environment_from_nested_request() {
        let temp = TempDir::new().unwrap();
        let bru_file = create_bruno_collection(&temp, "DC AI/Ask Agent");

        let env = load_environment(&bru_file, "LOCAL").unwrap();
        assert_eq!(env.vars.get("BASE_URL").unwrap(), "http://localhost:3000");
    }

    #[test]
    fn load_environment_not_found() {
        let temp = TempDir::new().unwrap();
        let bru_file = create_bruno_collection(&temp, "requests");

        let result = load_environment(&bru_file, "NONEXISTENT");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Environment file not found"));
    }

    fn make_bru(method: Method, url: &str, body: Option<Body>, headers: HashMap<String, String>) -> BruFile {
        BruFile {
            request: Request { method, url: url.to_string() },
            body,
            headers,
        }
    }

    #[test]
    fn apply_environment_substitutes_url() {
        let mut vars = HashMap::new();
        vars.insert("HOST".to_string(), "api.example.com".to_string());
        let env = Environment { vars };

        let mut bru = make_bru(Method::Get, "https://{{HOST}}/users", None, HashMap::new());

        apply_environment(&mut bru, &env);
        assert_eq!(bru.request.url, "https://api.example.com/users");
    }

    #[test]
    fn apply_environment_substitutes_body() {
        let mut vars = HashMap::new();
        vars.insert("TOKEN".to_string(), "secret123".to_string());
        let env = Environment { vars };

        let body = Body {
            body_type: "json".to_string(),
            content: r#"{"token": "{{TOKEN}}"}"#.to_string(),
        };
        let mut bru = make_bru(Method::Post, "https://api.example.com", Some(body), HashMap::new());

        apply_environment(&mut bru, &env);
        assert_eq!(bru.body.unwrap().content, r#"{"token": "secret123"}"#);
    }

    #[test]
    fn apply_environment_substitutes_headers() {
        let mut vars = HashMap::new();
        vars.insert("API_KEY".to_string(), "key123".to_string());
        let env = Environment { vars };

        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer {{API_KEY}}".to_string());
        let mut bru = make_bru(Method::Get, "https://api.example.com", None, headers);

        apply_environment(&mut bru, &env);
        assert_eq!(bru.headers.get("Authorization").unwrap(), "Bearer key123");
    }
}

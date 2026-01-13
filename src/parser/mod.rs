pub mod ast;

use std::collections::HashMap;
use ast::{BruFile, Request, Body, Method, Environment};

type CharIter<'a> = std::iter::Peekable<std::str::Chars<'a>>;

pub fn parse_bru_file(content: &str) -> Result<BruFile, String> {
    let mut request: Option<Request> = None;
    let mut body: Option<Body> = None;
    let mut headers: HashMap<String, String> = HashMap::new();

    let mut chars = content.chars().peekable();

    while chars.peek().is_some() {
        skip_whitespace(&mut chars);

        let block_name = read_identifier(&mut chars);
        if block_name.is_empty() {
            skip_line(&mut chars);
            continue;
        }

        skip_whitespace_no_newline(&mut chars);

        if block_name == "body" && chars.peek() == Some(&':') {
            chars.next();
            let body_type = read_identifier(&mut chars);
            skip_whitespace(&mut chars);
            if chars.peek() == Some(&'{') {
                chars.next();
                let content_str = read_balanced_braces(&mut chars);
                body = Some(Body {
                    body_type,
                    content: content_str.trim().to_string(),
                });
            }
            continue;
        }

        if chars.peek() != Some(&'{') {
            skip_line(&mut chars);
            continue;
        }
        chars.next();

        match block_name.as_str() {
            "get" | "post" | "put" | "delete" | "patch" | "options" | "head" => {
                request = Some(parse_method_block(&block_name, &mut chars)?);
            }
            "headers" => headers = parse_key_value_block(&mut chars),
            _ => skip_block(&mut chars),
        }
    }

    let request = request.ok_or("No request method block found")?;

    Ok(BruFile { request, body, headers })
}

pub fn parse_environment(content: &str) -> Result<Environment, String> {
    let mut vars: HashMap<String, String> = HashMap::new();
    let mut chars = content.chars().peekable();

    while chars.peek().is_some() {
        skip_whitespace(&mut chars);

        let block_name = read_identifier(&mut chars);
        if block_name.is_empty() {
            skip_line(&mut chars);
            continue;
        }

        skip_whitespace_no_newline(&mut chars);

        if chars.peek() != Some(&'{') {
            skip_line(&mut chars);
            continue;
        }
        chars.next();

        if block_name != "vars" {
            skip_block(&mut chars);
            continue;
        }
        vars = parse_key_value_block(&mut chars);
    }

    Ok(Environment { vars })
}

fn parse_method_block(method_str: &str, chars: &mut CharIter) -> Result<Request, String> {
    let entries = parse_key_value_block(chars);
    let method: Method = method_str.parse()?;
    let url = entries.get("url").cloned().unwrap_or_default();
    Ok(Request { method, url })
}

fn parse_key_value_block(chars: &mut CharIter) -> HashMap<String, String> {
    let mut result = HashMap::new();

    loop {
        skip_whitespace(chars);

        if chars.peek() == Some(&'}') {
            chars.next();
            break;
        }

        if chars.peek().is_none() {
            break;
        }

        let key = read_until_colon(chars);
        if key.is_empty() {
            skip_line(chars);
            continue;
        }

        if chars.peek() == Some(&':') {
            chars.next();
        }

        skip_whitespace_no_newline(chars);
        let value = read_line(chars);

        result.insert(key.trim().to_string(), value.trim().to_string());
    }

    result
}

fn read_balanced_braces(chars: &mut CharIter) -> String {
    let mut result = String::new();
    let mut depth = 1;

    while let Some(c) = chars.next() {
        match c {
            '{' => {
                depth += 1;
                result.push(c);
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
                result.push(c);
            }
            _ => result.push(c),
        }
    }

    result
}

fn skip_block(chars: &mut CharIter) {
    let mut depth = 1;
    while let Some(c) = chars.next() {
        match c {
            '{' => depth += 1,
            '}' if depth == 1 => break,
            '}' => depth -= 1,
            _ => {}
        }
    }
}

fn skip_whitespace(chars: &mut CharIter) {
    while chars.peek().is_some_and(|c| c.is_whitespace()) {
        chars.next();
    }
}

fn skip_whitespace_no_newline(chars: &mut CharIter) {
    while chars.peek().is_some_and(|&c| c == ' ' || c == '\t') {
        chars.next();
    }
}

fn skip_line(chars: &mut CharIter) {
    while let Some(c) = chars.next() {
        if c == '\n' {
            break;
        }
    }
}

fn read_identifier(chars: &mut CharIter) -> String {
    let mut result = String::new();
    while let Some(&c) = chars.peek() {
        if !c.is_alphanumeric() && c != '_' && c != '-' {
            break;
        }
        result.push(c);
        chars.next();
    }
    result
}

fn read_until_colon(chars: &mut CharIter) -> String {
    let mut result = String::new();
    while let Some(&c) = chars.peek() {
        if c == ':' || c == '\n' || c == '}' {
            break;
        }
        result.push(c);
        chars.next();
    }
    result
}

fn read_line(chars: &mut CharIter) -> String {
    let mut result = String::new();
    while let Some(c) = chars.next() {
        if c == '\n' {
            break;
        }
        result.push(c);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_get_request() {
        let content = r#"
get {
  url: https://api.example.com/users
}
"#;
        let bru = parse_bru_file(content).unwrap();
        assert_eq!(bru.request.url, "https://api.example.com/users");
        assert!(matches!(bru.request.method, Method::Get));
        assert!(bru.body.is_none());
    }

    #[test]
    fn parse_post_with_json_body() {
        let content = r#"
post {
  url: https://api.example.com/users
  body: json
}

body:json {
  {
    "name": "John",
    "age": 30
  }
}
"#;
        let bru = parse_bru_file(content).unwrap();
        assert_eq!(bru.request.url, "https://api.example.com/users");
        assert!(matches!(bru.request.method, Method::Post));
        assert!(bru.body.is_some());
        let body = bru.body.unwrap();
        assert_eq!(body.body_type, "json");
        assert!(body.content.contains("\"name\": \"John\""));
    }

    #[test]
    fn parse_request_with_headers() {
        let content = r#"
get {
  url: https://api.example.com/users
}

headers {
  Authorization: Bearer token123
  X-Custom-Header: custom-value
}
"#;
        let bru = parse_bru_file(content).unwrap();
        assert_eq!(bru.headers.get("Authorization").unwrap(), "Bearer token123");
        assert_eq!(bru.headers.get("X-Custom-Header").unwrap(), "custom-value");
    }

    #[test]
    fn parse_environment_vars() {
        let content = r#"
vars {
  API_URL: https://api.example.com
  API_KEY: secret123
}
"#;
        let env = parse_environment(content).unwrap();
        assert_eq!(env.vars.get("API_URL").unwrap(), "https://api.example.com");
        assert_eq!(env.vars.get("API_KEY").unwrap(), "secret123");
    }

    #[test]
    fn parse_nested_json_body() {
        let content = r#"
post {
  url: https://api.example.com
}

body:json {
  {
    "user": {
      "name": "John",
      "address": {
        "city": "NYC"
      }
    }
  }
}
"#;
        let bru = parse_bru_file(content).unwrap();
        let body = bru.body.unwrap();
        assert!(body.content.contains("\"address\""));
        assert!(body.content.contains("\"city\": \"NYC\""));
    }

    #[test]
    fn parse_all_http_methods() {
        for method in ["get", "post", "put", "delete", "patch", "options", "head"] {
            let content = format!(
                r#"
{} {{
  url: https://api.example.com
}}
"#,
                method
            );
            let bru = parse_bru_file(&content).unwrap();
            assert_eq!(bru.request.url, "https://api.example.com");
        }
    }

    #[test]
    fn error_on_missing_method_block() {
        let content = r#"
headers {
  Authorization: Bearer token
}
"#;
        let result = parse_bru_file(content);
        assert!(result.is_err());
    }
}

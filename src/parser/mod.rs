pub mod ast;

use std::collections::HashMap;
use ast::{BruFile, Request, Body, Method, Environment};

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

        if block_name == "body" {
            if chars.peek() == Some(&':') {
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
            _ => {
                skip_block(&mut chars);
            }
        }
    }

    let request = request.ok_or("No request method block found")?;

    Ok(BruFile {
        request,
        body,
        headers,
    })
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

        if block_name == "vars" {
            vars = parse_key_value_block(&mut chars);
        } else {
            skip_block(&mut chars);
        }
    }

    Ok(Environment { vars })
}

fn parse_method_block(
    method_str: &str,
    chars: &mut std::iter::Peekable<std::str::Chars>,
) -> Result<Request, String> {
    let entries = parse_key_value_block(chars);

    let method: Method = method_str.parse()?;
    let url = entries.get("url").cloned().unwrap_or_default();

    Ok(Request { method, url })
}

fn parse_key_value_block(
    chars: &mut std::iter::Peekable<std::str::Chars>,
) -> HashMap<String, String> {
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

fn read_balanced_braces(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut result = String::new();
    let mut depth = 1;

    while let Some(&c) = chars.peek() {
        if c == '{' {
            depth += 1;
            result.push(c);
            chars.next();
        } else if c == '}' {
            depth -= 1;
            if depth == 0 {
                chars.next();
                break;
            }
            result.push(c);
            chars.next();
        } else {
            result.push(c);
            chars.next();
        }
    }

    result
}

fn skip_block(chars: &mut std::iter::Peekable<std::str::Chars>) {
    let mut depth = 1;
    while let Some(c) = chars.next() {
        if c == '{' {
            depth += 1;
        } else if c == '}' {
            depth -= 1;
            if depth == 0 {
                break;
            }
        }
    }
}

fn skip_whitespace(chars: &mut std::iter::Peekable<std::str::Chars>) {
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
}

fn skip_whitespace_no_newline(chars: &mut std::iter::Peekable<std::str::Chars>) {
    while let Some(&c) = chars.peek() {
        if c == ' ' || c == '\t' {
            chars.next();
        } else {
            break;
        }
    }
}

fn skip_line(chars: &mut std::iter::Peekable<std::str::Chars>) {
    while let Some(&c) = chars.peek() {
        chars.next();
        if c == '\n' {
            break;
        }
    }
}

fn read_identifier(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut result = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_alphanumeric() || c == '_' || c == '-' {
            result.push(c);
            chars.next();
        } else {
            break;
        }
    }
    result
}

fn read_until_colon(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
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

fn read_line(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut result = String::new();
    while let Some(&c) = chars.peek() {
        if c == '\n' {
            chars.next();
            break;
        }
        result.push(c);
        chars.next();
    }
    result
}

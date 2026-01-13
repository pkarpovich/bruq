use crate::parser::ast::BruFile;

pub struct CurlOptions {
    pub verbose: bool,
    pub silent: bool,
}

impl Default for CurlOptions {
    fn default() -> Self {
        Self {
            verbose: false,
            silent: false,
        }
    }
}

pub fn generate_curl(bru: &BruFile, options: &CurlOptions) -> String {
    let mut parts: Vec<String> = vec!["curl".to_string()];

    if options.verbose {
        parts.push("-v".to_string());
    }

    if options.silent {
        parts.push("-s".to_string());
    }

    parts.push("-X".to_string());
    parts.push(bru.request.method.as_str().to_string());

    parts.push(format!("'{}'", bru.request.url));

    if let Some(ref body) = bru.body {
        let content_type = match body.body_type.as_str() {
            "json" => "application/json",
            "xml" => "application/xml",
            "text" => "text/plain",
            "form-urlencoded" => "application/x-www-form-urlencoded",
            _ => "application/json",
        };
        parts.push("-H".to_string());
        parts.push(format!("'Content-Type: {}'", content_type));
    }

    for (key, value) in &bru.headers {
        parts.push("-H".to_string());
        parts.push(format!("'{}: {}'", key, value));
    }

    if let Some(ref body) = bru.body {
        if !body.content.is_empty() {
            parts.push("-d".to_string());
            let escaped = escape_body(&body.content);
            parts.push(format!("'{}'", escaped));
        }
    }

    parts.join(" ")
}

fn escape_body(content: &str) -> String {
    content.replace('\'', "'\"'\"'")
}

use crate::parser::ast::BruFile;

#[derive(Default)]
pub struct CurlOptions {
    pub verbose: bool,
    pub silent: bool,
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

    let has_content_type = bru.headers.keys().any(|k| k.eq_ignore_ascii_case("content-type"));

    if let Some(ref body) = bru.body {
        if !has_content_type {
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

        if !body.content.is_empty() {
            parts.push("-d".to_string());
            parts.push(format!("'{}'", escape_body(&body.content)));
        }
    }

    for (key, value) in &bru.headers {
        parts.push("-H".to_string());
        parts.push(format!("'{}: {}'", key, value));
    }

    parts.join(" ")
}

fn escape_body(content: &str) -> String {
    content.replace('\'', "'\"'\"'")
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::{Body, Method, Request};
    use std::collections::HashMap;

    fn make_bru(method: Method, url: &str) -> BruFile {
        BruFile {
            request: Request {
                method,
                url: url.to_string(),
            },
            body: None,
            headers: HashMap::new(),
        }
    }

    #[test]
    fn simple_get_request() {
        let bru = make_bru(Method::Get, "https://api.example.com");
        let curl = generate_curl(&bru, &CurlOptions::default());
        assert_eq!(curl, "curl -X GET 'https://api.example.com'");
    }

    #[test]
    fn post_with_json_body() {
        let mut bru = make_bru(Method::Post, "https://api.example.com");
        bru.body = Some(Body {
            body_type: "json".to_string(),
            content: r#"{"name": "John"}"#.to_string(),
        });
        let curl = generate_curl(&bru, &CurlOptions::default());
        assert!(curl.contains("-X POST"));
        assert!(curl.contains("-H 'Content-Type: application/json'"));
        assert!(curl.contains(r#"-d '{"name": "John"}'"#));
    }

    #[test]
    fn request_with_headers() {
        let mut bru = make_bru(Method::Get, "https://api.example.com");
        bru.headers.insert("Authorization".to_string(), "Bearer token".to_string());
        let curl = generate_curl(&bru, &CurlOptions::default());
        assert!(curl.contains("-H 'Authorization: Bearer token'"));
    }

    #[test]
    fn verbose_and_silent_flags() {
        let bru = make_bru(Method::Get, "https://api.example.com");

        let curl_verbose = generate_curl(&bru, &CurlOptions { verbose: true, silent: false });
        assert!(curl_verbose.contains("-v"));

        let curl_silent = generate_curl(&bru, &CurlOptions { verbose: false, silent: true });
        assert!(curl_silent.contains("-s"));
    }

    #[test]
    fn no_duplicate_content_type() {
        let mut bru = make_bru(Method::Post, "https://api.example.com");
        bru.headers.insert("Content-Type".to_string(), "text/plain".to_string());
        bru.body = Some(Body {
            body_type: "json".to_string(),
            content: "test".to_string(),
        });
        let curl = generate_curl(&bru, &CurlOptions::default());
        let content_type_count = curl.matches("Content-Type").count();
        assert_eq!(content_type_count, 1);
    }

    #[test]
    fn escape_single_quotes_in_body() {
        let mut bru = make_bru(Method::Post, "https://api.example.com");
        bru.body = Some(Body {
            body_type: "json".to_string(),
            content: r#"{"name": "O'Brien"}"#.to_string(),
        });
        let curl = generate_curl(&bru, &CurlOptions::default());
        assert!(curl.contains("O'\"'\"'Brien"));
    }
}

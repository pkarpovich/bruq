use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BruFile {
    pub request: Request,
    pub body: Option<Body>,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct Body {
    pub body_type: String,
    pub content: String,
}

#[derive(Debug, Clone, Copy)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Options,
    Head,
}

impl Method {
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Patch => "PATCH",
            Method::Options => "OPTIONS",
            Method::Head => "HEAD",
        }
    }
}

impl std::str::FromStr for Method {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "get" => Ok(Method::Get),
            "post" => Ok(Method::Post),
            "put" => Ok(Method::Put),
            "delete" => Ok(Method::Delete),
            "patch" => Ok(Method::Patch),
            "options" => Ok(Method::Options),
            "head" => Ok(Method::Head),
            _ => Err(format!("Unknown method: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Environment {
    pub vars: HashMap<String, String>,
}

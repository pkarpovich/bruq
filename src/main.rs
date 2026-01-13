mod parser;
mod environment;
mod curl;

use std::fs;
use std::path::PathBuf;
use clap::Parser;

use crate::parser::parse_bru_file;
use crate::environment::{load_environment, substitute_variables};
use crate::curl::{generate_curl, CurlOptions};

#[derive(Parser)]
#[command(name = "bruq")]
#[command(about = "Convert Bruno .bru files to curl commands")]
struct Cli {
    #[arg(help = "Path to .bru file")]
    file: PathBuf,

    #[arg(short, long, help = "Environment name (looks in environments/<NAME>.bru)")]
    env: Option<String>,

    #[arg(short, long, help = "Include -v flag in curl output")]
    verbose: bool,

    #[arg(short, long, help = "Include -s flag in curl output")]
    silent: bool,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();

    let content = fs::read_to_string(&cli.file)
        .map_err(|e| format!("Cannot read file: {}", e))?;

    let mut bru = parse_bru_file(&content)?;

    if let Some(env_name) = &cli.env {
        let env = load_environment(&cli.file, env_name)?;
        bru.request.url = substitute_variables(&bru.request.url, &env.vars);

        if let Some(ref mut body) = bru.body {
            body.content = substitute_variables(&body.content, &env.vars);
        }

        for value in bru.headers.values_mut() {
            *value = substitute_variables(value, &env.vars);
        }
    }

    let options = CurlOptions {
        verbose: cli.verbose,
        silent: cli.silent,
    };

    let curl_cmd = generate_curl(&bru, &options);
    println!("{}", curl_cmd);

    Ok(())
}

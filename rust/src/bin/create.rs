use std::fmt::Debug;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, process};
use std::process::Command;
use clap::Parser;
use regex::Regex;

/// Create a new repository
#[derive(Parser)]
#[command(about)]
struct Cli {
    /// Path to repository
    #[arg(value_parser = clap::builder::ValueParser::new(parse_path))]
    path: PathBuf,
}

fn parse_path(path: &str) -> Result<PathBuf, String> {
    if path.len() > 256 {
        return Err(String::from("Path cannot exceed 256 characters"));
    }

    let parsed = Path::new(path).to_owned();
    if parsed.is_absolute() {
        return Err(String::from("Absolute paths are not allowed"));
    }
    if parsed.components().count() > 4 {
        return Err(String::from("Nesting must not exceed a depth of 4"));
    }

    let re = Regex::new("^(?:[A-Za-z0-9_\\-]+/)*[A-Za-z0-9_\\-]+(?:\\.git)?$").unwrap();
    if !re.is_match(path) {
        return Err(String::from("Repository and folder names can only contain alphanumeric \
        characters, hyphens, and underscores"));
    }

    let file_name = parsed.file_name().unwrap().to_str().unwrap();
    if file_name.ends_with(".git") {
        Ok(parsed)
    } else {
        Ok(parsed.with_file_name(format!("{}.git", file_name)))
    }
}

fn main() {
    let args = Cli::parse();

    let mut cmd = Command::new("git");
    cmd.arg("init").arg("--bare");

    if args.path.exists() {
        eprintln!("Repo already exists at '{}'", args.path.display());
        process::exit(1);
    }

    fs::create_dir_all(&args.path).expect("Failed to create folders");
    cmd.args(&args.path);
    cmd.output().expect("Failed to create repo");

    let mut repo_file = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open("repos")
        .unwrap();

    writeln!(repo_file, "{}", args.path.to_str().unwrap()).expect("Unable to write to repo file");
    println!("Repo created at '{}'", args.path.display());
}

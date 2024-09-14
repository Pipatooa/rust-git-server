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
    /// Name for repository
    #[arg(value_parser = clap::builder::ValueParser::new(parse_name))]
    name: String,
    /// Folder to file under
    #[arg(default_value = ".", hide_default_value = true,
    value_parser = clap::builder::ValueParser::new(parse_folder))]
    folder: PathBuf
}

fn parse_name(name: &str) -> Result<String, String> {
    let re = Regex::new("^[A-Za-z0-9_\\-]+(?:\\.git)?$").unwrap();
    if !re.is_match(&name) {
        return Err(String::from("Names can only contain alphanumeric characters, hyphens, and \
        underscores"));
    }

    if name.len() > 32 {
        return Err(String::from("Repository name cannot exceed 32 characters"));
    }

    if name.ends_with(".git") {
        Ok(name.to_owned())
    } else {
        Ok(format!("{}.git", name))
    }
}

fn parse_folder(path: &str) -> Result<PathBuf, String> {
    if path == "." {
        return Ok(PathBuf::from("."));
    }

    let re = Regex::new("^/?([A-Za-z0-9_\\-]+/)*[A-Za-z0-9_\\-]+/?$").unwrap();
    if !re.is_match(path) {
        return Err(String::from("Folder names can only contain alphanumeric characters, hyphens, \
        and underscores"));
    }

    let parsed = Path::new(path).to_owned();
    if parsed.is_absolute() {
        return Err(String::from("Absolute paths are not allowed"));
    }
    if parsed.components().count() > 4 {
        return Err(String::from("Nesting must not exceed a depth of 4"));
    }

    Ok(parsed)
}

fn main() {
    let args = Cli::parse();

    let mut cmd = Command::new("git");
    cmd.arg("init").arg("--bare");

    let path = args.folder.join(Path::new(args.name.as_str()));
    if path.exists() {
        eprintln!("Repo already exists at '{}'", path.display());
        process::exit(1);
    }

    fs::create_dir_all(&path).expect("Failed to create folders");
    cmd.args(path.to_str());
    cmd.output().expect("Failed to create repo");

    let mut repo_file = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open("repos")
        .unwrap();

    writeln!(repo_file, "{}", path.to_str().unwrap()).expect("Unable to write to repo file");
    println!("Repo created at '{}'", path.display());
}

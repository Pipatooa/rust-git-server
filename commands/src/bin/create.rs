use clap::Parser;
use commands::{get_repo_home, parse_repo_path};
use std::os::unix;
use std::path::PathBuf;
use std::process::Command;
use std::{fs, process};

/// Create a new repository
#[derive(Parser)]
#[command(about)]
struct Cli {
    /// Path to repository
    #[arg(value_parser = clap::builder::ValueParser::new(parse_repo_path))]
    path: PathBuf,
}

fn main() {
    let args = Cli::parse();

    let path = args.path;
    if path.exists() {
        eprintln!("Repo already exists at '{}'", path.display());
        process::exit(1);
    }

    let git_dir = get_repo_home().join(&path);
    fs::create_dir_all(&git_dir).expect("Failed to create folders");

    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder).expect("Failed to create folders");
    }

    Command::new("git")
        .args([
           "init",
            "--bare",
        ])
        .arg(&git_dir)
        .output()
        .expect("Failed to create repo");

    unix::fs::symlink(git_dir, &path).expect("Failed to link repo");
    println!("Repo created at '{}'", path.display());
}

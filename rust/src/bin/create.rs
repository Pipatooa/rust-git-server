use clap::Parser;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::{fs, process};
use commands::parse_path;

/// Create a new repository
#[derive(Parser)]
#[command(about)]
struct Cli {
    /// Path to repository
    #[arg(value_parser = clap::builder::ValueParser::new(parse_path))]
    path: PathBuf,
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

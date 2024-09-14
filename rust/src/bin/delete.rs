use clap::Parser;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::{fs, io, process};
use commands::parse_path;

/// Delete an existing repository
#[derive(Parser)]
#[command(about)]
struct Cli {
    /// Path to repository
    #[arg(value_parser = clap::builder::ValueParser::new(parse_path))]
    path: PathBuf,
    /// Disable confirmation prompt
    #[arg(long)]
    confirm: bool
}

fn main() {
    let args = Cli::parse();

    if !args.path.exists() {
        eprintln!("No repo exists at '{}'", args.path.display());
        process::exit(1);
    }

    if !args.confirm {
        let stdin = io::stdin();
        let input = &mut String::new();

        print!("Delete '{}' [y/N]: ", args.path.display());
        io::stdout().flush().unwrap();
        stdin.read_line(input).expect("Could not read input");

        match input.trim() {
            "y" => {},
            "Y" => {},
            _ => process::exit(1)
        }
    }

    let repo_file = OpenOptions::new()
        .read(true).open("repos")
        .expect("Failed to open repo file");
    let repo_file_reader = BufReader::new(repo_file);
    let repo_replacement = OpenOptions::new()
        .write(true).create(true).open("tmp_repos")
        .expect("Failed to create temporary repos file");
    let mut repo_replacement_writer = BufWriter::new(repo_replacement);

    let repo_path = args.path.to_str().unwrap();

    for line in repo_file_reader.lines() {
        let line = line.expect("Failed to read from repo file");
        if !line.eq(repo_path) {
            writeln!(repo_replacement_writer, "{}", line).expect("Failed to write to temporary \
            repo file");
        }
    }

    fs::remove_dir_all(&args.path).expect("Failed to remove repository");
    fs::rename("tmp_repos", "repos").expect("Failed to update repo file");
    println!("Deleted '{}'", args.path.display());
}

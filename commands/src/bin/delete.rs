use clap::Parser;
use commands::{clean_empty_parent_folders, get_repo_home, parse_repo_path};
use std::io::Write;
use std::path::PathBuf;
use std::{fs, io, process};

/// Delete an existing repository
#[derive(Parser)]
#[command(about)]
struct Cli {
    /// Path to repository
    #[arg(value_parser = clap::builder::ValueParser::new(parse_repo_path))]
    path: PathBuf,
    /// Disable confirmation prompt
    #[arg(long)]
    confirm: bool
}

fn main() {
    let args = Cli::parse();

    let path = args.path;
    if !path.exists() {
        eprintln!("No repo exists at '{}'", path.display());
        process::exit(1);
    }

    if !args.confirm {
        let stdin = io::stdin();
        let input = &mut String::new();

        print!("Delete '{}' [y/N]: ", path.display());
        io::stdout().flush().unwrap();
        stdin.read_line(input).expect("Could not read input");

        match input.trim() {
            "y" => {},
            "Y" => {},
            _ => process::exit(1)
        }
    }

    let repo_home = get_repo_home();
    let git_dir = repo_home.join(&path);
    fs::remove_file(&path).expect("Failed to unlink repo");
    fs::remove_dir_all(&git_dir).expect("Failed to remove repo");

    clean_empty_parent_folders(&path, None);
    clean_empty_parent_folders(&git_dir, Some(&repo_home));

    println!("Deleted '{}'", path.display());
}

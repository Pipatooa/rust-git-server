use clap::Parser;
use std::{process};
use std::path::Path;
use std::process::Command;
use regex::Regex;

/// List all commands or get help for a specific command
#[derive(Parser)]
#[command(about)]
struct Cli {
    /// Command to get help for
    #[arg(value_parser = clap::builder::ValueParser::new(parse_command))]
    command: Option<String>
}

fn parse_command(command: &str) -> Result<String, String> {
    let re = Regex::new("^[a-z]+$").unwrap();
    if !re.is_match(&command) {
        return Err(String::from("Invalid command"));
    }
    Ok(command.to_string())
}

fn main() {
    let args = Cli::parse();

    if args.command.is_none() {
        println!(" \
Available commands:
  help   : list all commands or get help for a specific command
  create : create a new repository
  delete : delete an existing repository
  rename : rename a single existing repository
  move   : move all matching repositories into a folder
  list   : list all repositories matching any filters
  keys   : edit authorized_keys"
        );
        return;
    }

    let command = args.command.unwrap();
    let path = Path::new("git-shell-commands").join(&command);
    if !path.is_file() {
        eprintln!("No such command '{}'", command);
        process::exit(1);
    }

    let result = Command::new(path)
        .arg("--help")
        .spawn();
    match result {
        Ok(mut child)=> child.wait().ok(),
        Err(_) => {
            eprintln!("Failed to get help for '{}'", command);
            process::exit(1);
        }
    };
}

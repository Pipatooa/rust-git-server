use clap::Parser;
use commands::parse_command;
use std::path::Path;
use std::process;
use std::process::Command;

/// List all commands or get help for a specific command
#[derive(Parser)]
#[command(about)]
struct Cli {
    /// Command to get help for
    #[arg(value_parser = clap::builder::ValueParser::new(parse_command))]
    command: Option<String>,
}

fn main() {
    let args = Cli::parse();

    if args.command.is_none() {
        println!(
            " \
Available commands:
  help    : list all commands or get help for a specific command
  aliases : list all aliases for a command

  create  : create new repositories
  delete  : delete existing repositories
  move    : rename a single or move multiple repositories
  list    : list all repositories matching any filters"
        );
        return;
    }

    let command = args.command.unwrap();
    let path = Path::new("git-shell-commands").join(&command);
    if !path.is_file() {
        eprintln!("No such command '{}'", command);
        process::exit(1);
    }

    let result = Command::new(path).arg("--help").spawn();
    match result {
        Ok(mut child) => child.wait().ok(),
        Err(_) => {
            eprintln!("Failed to get help for '{}'", command);
            process::exit(1);
        }
    };
}

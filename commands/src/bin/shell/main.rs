mod git;
mod interactive;
mod commands;

use clap::builder::ValueParser;
use clap::Parser;

/// Limited shell
#[derive(Parser)]
#[command(about, disable_help_flag = true)]
struct Cli {
    /// Command to run
    #[arg(short, value_parser = ValueParser::new(parse_command))]
    command: Option<String>,
}

fn parse_command(command: &str) -> Result<String, String> {
    if command.is_empty() {
        Err(String::from("command must not be empty"))
    } else {
        Ok(command.to_string())
    }
}

fn main() {
    let args = Cli::parse();
    if args.command.is_none() {
        interactive::interactive_shell();
        return;
    }

    let command = &args.command.unwrap();
    let args = shlex::split(command).expect("Failed to split command");

    if args.is_empty() {
        std::process::exit(1);
    }

    match commands::invoke_command(None, &args) {
        Ok(exit_code) => std::process::exit(exit_code),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

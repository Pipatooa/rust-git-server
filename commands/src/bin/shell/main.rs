mod git;
mod interactive;
mod invoke;

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
        eprintln!("No command specified");
        std::process::exit(1);
    }

    let command = &args[0];
    let args = &args[1..];

    match command.as_str() {
        "git-receive-pack"   => git::git_receive_pack(args),
        "git-upload-pack"    => git::git_receive_pack(args),
        "git upload-archive" => git::git_upload_archive(args),
        _ => (),
    }

    match invoke::invoke_command(command, args) {
        Ok(Some(status)) => std::process::exit(status.code().unwrap()),
        Ok(None) => std::process::exit(1),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

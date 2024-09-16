use clap::Parser;
use commands::parse_command;
use std::path::PathBuf;

/// List aliases for a command
#[derive(Parser)]
#[command(about)]
struct Cli {
    /// Command
    #[arg(value_parser = clap::builder::ValueParser::new(parse_command))]
    command: String,
}

fn main() {
    let args = Cli::parse();

    let command_dir = PathBuf::from("git-shell-commands");
    let command = command_dir.join(args.command);

    let base_command = if command.is_symlink() {
        command.read_link().unwrap()
    } else if command.is_file() {
        command.strip_prefix("git-shell-commands").unwrap().to_path_buf()
    } else {
        eprintln!("Command not found");
        std::process::exit(1);
    };

    let aliases = command_dir.read_dir().expect("Could not read commands")
        .filter_map(Result::ok)
        .filter_map(|entry| {
            if entry.file_type().unwrap().is_symlink() {
                let path = entry.path();
                let dst = path.read_link().unwrap();
                if dst.eq(&base_command) {
                    return Some(path.file_name().unwrap().to_str().unwrap().to_string())
                }
            }
            None
        }).collect::<Vec<_>>();

    let base_command_name = base_command.file_name().unwrap().to_str().unwrap();
    println!("Aliases for command '{}':", base_command_name);
    println!("  {}", aliases.join(", "));
}

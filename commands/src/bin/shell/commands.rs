use std::io;
use crossterm::terminal;
use crossterm::terminal::ClearType;
use phf::phf_map;
use crate::{git, interactive};
use crate::interactive::Shell;

mod delete;
mod create;
mod list;
mod r#move;

static ALIASES: phf::Map<&'static str, &'static str> = phf_map! {
    "mk" => "add",
    "init" => "add",
    "rm" => "delete",
    "remove" => "delete",
    "del" => "delete",
    "mv" => "move",
    "rename" => "move",
    "ls" => "list",
    "l" => "list",
    "dir" => "list",
    "find" => "list"
};

pub fn invoke_command(shell: Option<&mut Shell>, args: &Vec<String>) -> io::Result<i32> {
    let toggle_raw = shell.is_some();
    if toggle_raw {
        terminal::disable_raw_mode()?;
    }

    let command = &args[0].as_str();
    let de_aliased = ALIASES.get(command).unwrap_or(command);

    let result = match (shell, de_aliased.as_ref()) {
        (None, "git-receive-pack")   => git::git_receive_pack(args),
        (None, "git-upload-pack")    => git::git_upload_pack(args),
        (None, "git upload-archive") => git::git_upload_archive(args),
        (Some(_), "exit") => interactive::exit_interactive(0),
        (Some(s), "clear") => interactive::prompt_clear(s, ClearType::Purge),
        (_, "create") => create::invoke(args),
        (_, "delete") => delete::invoke(args),
        (_, "move") => r#move::invoke(args),
        (_, "list") => list::invoke(args),
        _ => {
            eprintln!("{}: command not found...", command);
            Ok(127)
        }
    };

    if toggle_raw {
        terminal::enable_raw_mode()?;
    }

    result
}

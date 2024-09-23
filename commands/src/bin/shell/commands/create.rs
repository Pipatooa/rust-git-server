use clap::Parser;
use commands::{get_repo_home, parse_repo_path};
use itertools::Itertools;
use std::collections::HashSet;
use std::os::unix;
use std::path::PathBuf;
use std::process::Command;
use std::{fs, io};

/// Create new repositories
#[derive(Parser, Debug)]
#[command(about, arg_required_else_help = true)]
struct Cli {
    /// Paths to repositories
    #[arg(required = true, num_args = 1.., value_parser = clap::builder::ValueParser::new(parse_repo_path))]
    path: Vec<PathBuf>,
}

pub fn invoke(args: &Vec<String>) -> io::Result<i32> {
    let result = Cli::try_parse_from(args);
    match result {
        Err(e) => { eprintln!("{}", e); Ok(e.exit_code()) },
        Ok(args) => Ok(run(args)),
    }
}

fn run(args: Cli) -> i32 {
    let paths = args.path.iter().collect::<HashSet<_>>();
    let existing = paths.iter().filter(|p| p.exists()).collect::<Vec<_>>();

    if existing.len() > 0 {
        match existing.len() {
            1 => eprintln!("Repo already exists at '{}'", existing[0].display()),
            _ => eprintln!(
                "Repos already exist at: {}",
                existing
                    .iter()
                    .map(|p| format!("'{}'", p.display()))
                    .join(", ")
            ),
        }
        return 1;
    }

    let git_home = get_repo_home();
    for path in &paths {
        let git_dir = git_home.join(&path);
        fs::create_dir_all(&git_dir).expect("Failed to create folders");

        if let Some(folder) = path.parent() {
            fs::create_dir_all(folder).expect("Failed to create folders");
        }

        Command::new("git")
            .args(["init", "--bare"])
            .arg(&git_dir)
            .output()
            .expect("Failed to create repo");

        unix::fs::symlink(git_dir, &path).expect("Failed to link repo");
        println!("Created '{}'", path.display());
    }
    if paths.len() > 1 {
        println!("Created {} new repositories", paths.len());
    }
    0
}
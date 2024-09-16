use clap::Parser;
use commands::{
    clean_empty_parent_folders, filter_repos, get_repo_home, make_glob_set, parse_repo_glob,
    represents_repo,
};
use globset::Glob;
use std::io::Write;
use std::{fs, io};

/// Delete existing repositories
#[derive(Parser)]
#[command(about, arg_required_else_help = true)]
struct Cli {
    /// Paths to repositories
    #[arg(required = true, num_args = 1.., value_parser = clap::builder::ValueParser::new(parse_repo_glob))]
    path: Vec<Glob>,
    /// List changes without performing them
    #[arg(group = "dry", short, long)]
    dry_run: bool,
    /// Disable confirmation prompt
    #[arg(group = "dry", long)]
    confirm: bool,
}

fn main() {
    let args = Cli::parse();

    let glob_set = make_glob_set(args.path.iter());
    let paths = filter_repos(None, true, |path| glob_set.is_match(path))
        .map(|path| {
            if represents_repo(&path) {
                vec![path]
            } else {
                filter_repos(Some(path), false, |_| true).collect::<Vec<_>>()
            }
        })
        .flatten()
        .collect::<Vec<_>>();

    if paths.is_empty() {
        eprintln!("No matching repositories found");
        std::process::exit(1);
    }

    if args.dry_run {
        println!("Plan to delete {} repositories:", paths.len());
        for path in paths {
            println!("Delete '{}'", path.display());
        }
        return;
    }

    let mut deleted = 0;
    for path in &paths {
        if !args.confirm {
            let stdin = io::stdin();
            let input = &mut String::new();

            print!("Delete '{}' [y/N]: ", path.display());
            io::stdout().flush().unwrap();
            stdin.read_line(input).expect("Could not read input");

            match input.trim() {
                "y" => {}
                "Y" => {}
                _ => continue,
            }
        }

        let repo_home = get_repo_home();
        let git_dir = repo_home.join(&path);
        fs::remove_file(&path).expect("Failed to unlink repo");
        fs::remove_dir_all(&git_dir).expect("Failed to remove repo");

        clean_empty_parent_folders(&path, None);
        clean_empty_parent_folders(&git_dir, Some(&repo_home));

        println!("Deleted '{}'", path.display());
        deleted += 1;
    }
    match deleted {
        1 => println!("Deleted 1 repository"),
        _ => println!("Deleted {} repositories", deleted),
    }
}

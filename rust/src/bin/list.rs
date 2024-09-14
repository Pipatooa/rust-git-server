use clap::Parser;
use std::fs;
use std::io::{stdout, Write};

/// List all repositories matching any filters
#[derive(Parser)]
#[command(about)]
struct Cli {
    /// Filter to apply to results
    filter: Option<String>,
    #[arg(short, long, default_value_t = false, requires("filter"))]
    /// Invert filter
    invert: bool,
}

fn match_repo(repo_name: &str, args: &Cli) -> bool {
    let filter = match args.filter {
        Some(ref str) => str,
        None => return !args.invert
    };

    repo_name.starts_with(filter) ^ args.invert
}

fn main() {
    let args = Cli::parse();

    let repo_list = fs::read_to_string("repos").expect("Failed to open repo file");

    let mut count: u32 = 0;
    let mut total_count: u32 = 0;

    let mut lock = stdout().lock();
    for repo_name in repo_list.lines() {
        total_count += 1;
        if match_repo(repo_name, &args) {
            count += 1;
            println!("{}", repo_name);
        }
    }
    lock.flush().unwrap();

    match (count, total_count) {
        (_, 0) => println!("You have no repositories."),
        (0, _) => println!("No results match filter."),
        _      => println!("Matched {} repositories out of {} total.", count, total_count)
    }
}

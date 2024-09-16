use clap::Parser;
use commands::filter_repos;

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

    let mut count: u32 = 0;
    let mut total_count: u32 = 0;

    for path in filter_repos(false, |_| true) {
        total_count += 1;

        if match_repo(path.to_str().unwrap(), &args) {
            count += 1;
            println!("{}", path.display());
        }
    }

    match (count, total_count) {
        (_, 0) => println!("You have no repositories."),
        (0, _) => println!("No results match filter."),
        _      => println!("Matched {} repositories out of {} total.", count, total_count)
    }
}

use clap::Parser;
use commands::{filter_repos, make_glob_set, parse_repo_glob};
use globset::Glob;
use std::iter;

/// List all repositories matching any filters
#[derive(Parser)]
#[command(about)]
struct Cli {
    /// Filters to apply to results
    #[arg(num_args = 0.., value_parser = clap::builder::ValueParser::new(parse_repo_glob))]
    filter: Option<Vec<Glob>>,
    #[arg(short, long, default_value_t = false, requires("filter"))]
    /// Invert filter
    invert: bool,
    /// Only output number of matches
    #[arg(short, long)]
    count: bool,
}

fn main() {
    let args = Cli::parse();

    let glob_set = match args.filter {
        Some(set) => make_glob_set(set.iter()),
        None => make_glob_set(iter::empty()),
    };

    let mut count: u32 = 0;
    let mut total_count: u32 = 0;

    for path in filter_repos(None, false, |_| true) {
        total_count += 1;

        if args.invert ^ (glob_set.is_empty() || glob_set.is_match(&path)) {
            count += 1;
            if !args.count {
                println!("{}", path.display());
            }
        }
    }

    if args.count {
        println!("{}", count);
        return;
    }

    match (count, total_count) {
        (_, 0) => println!("You have no repositories"),
        (0, _) => println!("Matched no repositories ({} total)", total_count),
        _ => println!("Matched {}/{} repositories", count, total_count),
    }
}

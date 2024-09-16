use clap::Parser;
use commands::{can_represent_repo, clean_empty_parent_folders, enforce_git_suffix, filter_repos, get_repo_home, parse_repo_glob, represents_repo};
use commands::{make_glob_set, parse_repo_path_or_folder};
use globset::Glob;
use itertools::{Either, Itertools};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::os::unix;
use std::path::PathBuf;

/// Rename a single or move multiple repositories
#[derive(Parser)]
#[command(about, arg_required_else_help = true)]
struct Cli {
    /// Paths to repositories
    #[arg(num_args = 1.., value_parser = clap::builder::ValueParser::new(parse_repo_glob))]
    source: Vec<Glob>,
    #[arg(short, long)]
    filter: Option<String>,
    /// New path to repository
    #[arg(value_parser = clap::builder::ValueParser::new(parse_repo_path_or_folder))]
    destination: PathBuf,
    /// List changes without performing them
    #[arg(short, long)]
    dry_run: bool
}

fn main() {
    let args = Cli::parse();

    let glob_set = make_glob_set(args.source.iter());
    let sources = filter_repos(None, true, |path| glob_set.is_match(path))
        .collect::<Vec<_>>();

    if sources.is_empty() {
        eprintln!("No matching repositories found");
        std::process::exit(1);
    }

    let rename_only = sources.len() == 1
        && represents_repo(&sources[0])
        && can_represent_repo(&args.destination);

    if rename_only {
        move_single(&sources[0], &args.destination, args.dry_run);
    } else {
        move_multiple(&sources, &args.destination, args.dry_run);
    }
}

fn move_single(src: &PathBuf, dst: &PathBuf, dry_run: bool) {
    let dst = enforce_git_suffix(dst.to_path_buf()).unwrap();

    if src.eq(&dst) {
        println!("Nothing to do");
        return;
    }

    if dst.exists() {
        eprintln!("Cannot rename '{}' -> '{}' : Destination occupied", src.display(), dst.display());
        std::process::exit(1);
    }

    println!("'{}' -> '{}'", src.display(), dst.display());
    if dry_run {
        return;
    }

    let repo_home = get_repo_home();
    let git_src = repo_home.join(&src);
    let git_dst = repo_home.join(&dst);

    fs::create_dir_all(&git_dst).expect("Failed to create folders");

    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).expect("Failed to create folders");
    }

    fs::rename(&git_src, &git_dst).expect("Failed to move repository");
    unix::fs::symlink(git_dst, dst).expect("Failed to link repo");

    fs::remove_file(&src).expect("Failed to unlink old location");

    clean_empty_parent_folders(&src, None);
    clean_empty_parent_folders(&git_src, Some(&repo_home));
    println!("Repository renamed");
}

fn move_multiple(sources: &Vec<PathBuf>, dst: &PathBuf, dry_run: bool) {
    if represents_repo(dst) {
        eprintln!("Destination is a repository, but multiple sources match");
        std::process::exit(1);
    }

    let moves = sources.iter().map(|path| {
        if represents_repo(path) {
            let name = path.file_name().unwrap().to_str().unwrap();
            vec![(path.to_path_buf(), dst.join(name))]
        } else {
            let path_parent = path.parent();
            filter_repos(Some(path.to_owned()), false, |_| true)
                .map(|src| {
                    let dst = if let Some(parent) = path_parent {
                        let clone = src.clone();
                        let relative = clone.strip_prefix(parent).unwrap();
                        dst.join(relative)
                    } else {
                        dst.join(&src)
                    };
                    (src, dst)
                })
                .collect::<Vec<_>>()
        }
    }).flatten().filter(|(src, dst)| !src.eq(dst)).collect::<Vec<_>>();

    let mut tmp_required = false;
    let mut destinations_from: HashMap<PathBuf, PathBuf> = HashMap::new();
    let mut not_replaced: HashSet<&PathBuf> = HashSet::from_iter(moves.iter().map(|(src, _)| src));

    let (moves, problems): (Vec<_>, Vec<_>) = moves.iter()
        .partition_map(|(src, dst)| {
            if dst.exists() {
                if !moves.iter().any(|(src, _)| src.eq(dst)) {
                    return Either::Right(
                        (src, dst, String::from("Destination occupied by unmoved repository"))
                    );
                }
                tmp_required = true;
            }
            if dst.components().count() > 4 {
                return Either::Right(
                    (src, dst, String::from("Destination has a nesting deeper than 4"))
                );
            }
            if let Some(conflict) = destinations_from.get(dst) {
                return Either::Right(
                    (src, dst, format!("Overlapping destination : '{}' -> '{}'",
                                       conflict.display(), dst.display()).as_str().to_owned())
                );
            }
            destinations_from.insert(dst.clone(), src.to_path_buf());
            not_replaced.remove(dst);
            Either::Left((src, dst))
        });

    if problems.len() > 0 {
        println!("Able to move {} repositories:", moves.len());
        for (src, dst) in moves {
            println!("'{}' -> '{}'", src.display(), dst.display());
        }
        if problems.len() == 1 {
            eprintln!("1 problem:");
        } else {
            eprintln!("{} problems:", problems.len());
        }
        eprintln!("{}", problems.iter().map(|(src, dst, err)|
            format!("'{}' -> '{}' : {}", src.display(), dst.display(), err)).join("\n"));
        std::process::exit(1);
    }

    if moves.is_empty() {
        eprintln!("Nothing to do");
        return;
    }

    if dry_run {
        println!("Plan to move {} repositories:", moves.len());
        for (src, dst) in moves {
            println!("'{}' -> '{}'", src.display(), dst.display());
        }
        return;
    }

    const TMP_DIR: &str = ".tmp";
    let repo_home = get_repo_home();
    let tmp_home = repo_home.join(TMP_DIR);

    for (_, dst) in &moves {
        let git_dst = repo_home.join(&dst);
        fs::create_dir_all(&git_dst).expect("Failed to create folders");
    }

    for (_, dst) in &moves {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).expect("Failed to create folders");
        }
    }

    let repo_src = if !tmp_required {
        &repo_home
    } else {
        for (src, _) in &moves {
            let tmp_git_dst = tmp_home.join(&src);
            let git_dst = repo_home.join(&src);
            fs::create_dir_all(&tmp_git_dst).expect("Failed to create folders");
            fs::create_dir_all(&git_dst).expect("Failed to create folders");
        }
        for (src, _) in &moves {
            let git_src = repo_home.join(&src);
            let tmp_git_dst = tmp_home.join(&src);
            fs::rename(&git_src, &tmp_git_dst).expect("Failed to move repository");
        }
        &tmp_home
    };

    for (src, dst) in &moves {
        let git_src = repo_src.join(&src);
        let git_dst = repo_home.join(&dst);
        println!("'{}' -> '{}'", src.display(), dst.display());
        fs::rename(&git_src, &git_dst).expect("Failed to move repository");
    }

    for (_, dst) in &moves {
        let git_dst = repo_home.join(&dst);
        if dst.is_symlink() {
            fs::remove_file(&dst).expect("Failed to unlink old location");
        }
        unix::fs::symlink(git_dst, dst).expect("Failed to link repo");
    }

    for src in &not_replaced {
        fs::remove_file(&src).expect("Failed to unlink old location");
    }

    for src in &not_replaced {
        let git_src = repo_home.join(&src);
        clean_empty_parent_folders(&src, None);
        clean_empty_parent_folders(&git_src, None);
    }

    if tmp_required {
        fs::remove_dir_all(tmp_home).expect("Failed to remove temp folder");
    }

    println!("Moved {} repositories", moves.len());
}
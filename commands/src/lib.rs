use globset::{Glob, GlobBuilder};
use regex::Regex;
use std::fs::FileType;
use std::path::{Path, PathBuf};
use std::{fs, iter};
use users::get_current_username;
use walkdir::{IntoIter, WalkDir};

pub const BASE_REPO_PATH: &str = "/srv/repos";

pub fn get_repo_home() -> PathBuf {
    let username = get_current_username().expect("Failed to get current username");
    Path::new(BASE_REPO_PATH).join(username)
}

pub fn clean_empty_parent_folders(path: &Path, stop: Option<&Path>) {
    let mut current = path;
    while let Some(parent) = current.parent() {
        current = parent;
        if !current.exists() || stop.is_some_and(|s| current.eq(s)) {
            break;
        }
        if fs::read_dir(current).expect("Failed to read directory contents").next().is_none() {
            fs::remove_dir(current).expect("Failed to remove empty parent")
        } else {
            break;
        }
    }
}

pub fn represents_repo(path: &Path) -> bool {
    path.to_str().unwrap().ends_with(".git")
}

pub fn can_represent_repo(path: &Path) -> bool {
    let raw = path.to_str().unwrap();
    !raw.is_empty() && !raw.ends_with('/')
}

pub fn enforce_git_suffix(path: PathBuf) -> Result<PathBuf, String> {
    let raw = path.to_str().unwrap();
    if raw.is_empty() || raw.ends_with('/') {
        Err(String::from("Directory not allowed"))
    } else if raw.ends_with(".git") {
        Ok(path)
    } else {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        Ok(path.with_file_name(format!("{}.git", file_name)))
    }
}

pub fn filter_repos<F>(match_folders: bool, mut filter: F) -> impl Iterator<Item = PathBuf>
where F: FnMut(&Path) -> bool {
    FilterRepos::new(None, match_folders, move |path: &Path, file_type: FileType| {
        let matched = filter(path);
        (matched, matched && match_folders && file_type.is_dir())
    })
}

pub fn t_filter_repos<F>(root: Option<PathBuf>, match_folders: bool, mut filter: F) -> impl Iterator<Item = PathBuf>
where F: FnMut(&Path) -> bool {
    FilterRepos::new(root, match_folders, move |path: &Path, file_type: FileType| {
        let matched = filter(path);
        (matched, matched && match_folders && file_type.is_dir())
    })
}

#[derive(Debug)]
pub struct FilterRepos<I, P> {
    it: I,
    root: Option<PathBuf>,
    match_folders: bool,
    predicate: P
}

impl<P> FilterRepos<IntoIter, P>
where P: FnMut(&Path, FileType) -> (bool, bool) {
    fn new(root: Option<PathBuf>, match_folders: bool, predicate: P) -> impl Iterator<Item=PathBuf> + Sized {
        let it_root = match &root {
            Some(root) => root,
            None => &PathBuf::from(".")
        };
        let it = WalkDir::new(it_root).into_iter();
        FilterRepos { it, root, match_folders, predicate }
    }
}

impl<P> Iterator for FilterRepos<IntoIter, P>
where P: FnMut(&Path, FileType) -> (bool, bool) {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.it.next();
            let entry = match next {
                None => return None,
                Some(Ok(entry)) => entry,
                Some(Err(err)) => {
                    eprintln!("Error traversing repos: {}", err);
                    std::process::exit(1)
                }
            };

            let components = entry.path().components();
            let base_folder = components.skip(1).next();
            match base_folder {
                None => continue,
                Some(component) => match component.as_os_str().to_str()? {
                    ".ssh"               => continue,
                    "git-shell-commands" => continue,
                    _ => ()
                }
            };

            if self.root.is_none() {
                match base_folder?.as_os_str().to_str()? {
                    ".ssh"               => continue,
                    "git-shell-commands" => continue,
                    _ => ()
                }
            }

            let file_type = entry.file_type();
            match (self.match_folders, file_type.is_symlink(), file_type.is_dir()) {
                (_, true, _) => (),
                (true, _, true) => (),
                _               => continue
            }

            let path = match &self.root.is_some() {
                true  => entry.path(),
                false => entry.path().strip_prefix(".").unwrap()
            };

            let (matches, skip_dir) = (self.predicate)(&path, file_type);
            if skip_dir {
                self.it.skip_current_dir();
            }
            if matches {
                return Some(path.to_owned());
            }
        }
    }
}

impl<P> iter::FusedIterator for FilterRepos<IntoIter, P>
where P: FnMut(&Path, FileType) -> (bool, bool) {}

pub fn parse_command(command: &str) -> Result<String, String> {
    if command.is_empty() {
        return Err(String::from("Command cannot be empty"));
    }

    let re = Regex::new("^[a-z]+$").unwrap();
    if !re.is_match(&command) {
        return Err(String::from("Invalid command"));
    }
    Ok(command.to_string())
}

pub fn parse_repo_path(path: &str) -> Result<PathBuf, String> {
    match parse_repo_path_or_folder(path) {
        Err(e) => Err(e),
        Ok(parsed) => enforce_git_suffix(parsed)
    }
}

pub fn parse_repo_path_or_folder(path: &str) -> Result<PathBuf, String> {
    match path.len() {
        0 => return Err(String::from("Path cannot be empty")),
        257.. => return Err(String::from("Path cannot exceed 256 characters")),
        _ => ()
    }

    if path == "." {
        return Ok(PathBuf::new())
    }

    let parsed = Path::new(path).to_owned();
    if parsed.is_absolute() {
        return Err(String::from("Absolute paths are not allowed"));
    }
    if parsed.components().any(|c| c.as_os_str().eq("..")) {
        return Err(String::from("Backtracking not allowed"));
    }
    if parsed.components().count() > 4 {
        return Err(String::from("Nesting must not exceed a depth of 4"));
    }

    let re = Regex::new("^(?:[A-Za-z0-9_\\-]+/)*[A-Za-z0-9_\\-]+(?:\\.git|/)?$").unwrap();
    if !re.is_match(path) {
        return Err(String::from("Repository and folder names can only contain alphanumeric \
    characters, hyphens, and underscores"));
    }
    if path.starts_with("git-shell-commands/") {
        return Err(String::from("Folder name disallowed"));
    }

    Ok(parsed)
}

pub fn parse_repo_glob(glob: &str) -> Result<Glob, String> {
    match glob.len() {
        0 => return Err(String::from("Glob cannot be empty")),
        65.. => return Err(String::from("Glob cannot exceed 64 characters")),
        _ => ()
    }

    let path = Path::new(glob);
    if path.is_absolute() {
        return Err(String::from("Absolute paths are not allowed"));
    }
    if path.components().any(|c| c.as_os_str().eq("..")) {
        return Err(String::from("Backtracking not allowed"));
    }

    let s: String;
    let glob = if glob == "." {
        "**/*"
    } else if glob.ends_with(".git") {
        glob
    } else if glob.ends_with('/') {
        glob.strip_suffix('/').unwrap()
    } else if glob.ends_with("**") {
        s = format!("{}/", glob);
        s.as_str()
    } else {
        s = format!("{}.git", glob);
        s.as_str()
    };

    let parsed = GlobBuilder::new(glob)
        .literal_separator(true)
        .build();
    match parsed {
        Ok(glob) => Ok(glob),
        Err(e) => Err(e.to_string())
    }
}

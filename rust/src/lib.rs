use std::path::{Path, PathBuf};
use regex::Regex;

pub fn parse_path(path: &str) -> Result<PathBuf, String> {
    match path.len() {
        0 => return Err(String::from("Path cannot be empty")),
        257.. => return Err(String::from("Path cannot exceed 256 characters")),
        _ => {}
    }

    let parsed = Path::new(path).to_owned();
    if parsed.is_absolute() {
        return Err(String::from("Absolute paths are not allowed"));
    }
    if parsed.components().count() > 4 {
        return Err(String::from("Nesting must not exceed a depth of 4"));
    }

    let re = Regex::new("^(?:[A-Za-z0-9_\\-]+/)*[A-Za-z0-9_\\-]+(?:\\.git)?$").unwrap();
    if !re.is_match(path) {
        return Err(String::from("Repository and folder names can only contain alphanumeric \
        characters, hyphens, and underscores"));
    }
    if path.starts_with("git-shell-commands/") {
        return Err(String::from("Folder name disallowed"));
    }

    let file_name = parsed.file_name().unwrap().to_str().unwrap();
    if file_name.ends_with(".git") {
        Ok(parsed)
    } else {
        Ok(parsed.with_file_name(format!("{}.git", file_name)))
    }
}

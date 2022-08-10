#![allow(dead_code, unused_variables)]

use std::path::Path;
use std::process::Command;

fn get_github_url(filepath: &Path, line: usize) -> Option<String> {
    let local_repo = get_local_repo_path()?;
    let github_repo = get_github_repo()?;

    None
}

/// runs `git remote -v` and checks if it's a github URL
fn get_github_repo() -> Option<String> {
    let res = Command::new("git").args(&["remote", "-v"]).output().ok()?;

    if !res.status.success() {
        return None;
    }

    let res = String::from_utf8(res.stdout).ok()?;

    None
}

/// runs `git rev-parse --show-toplevel` to get filepath of root
pub fn get_local_repo_path() -> Option<String> {
    let res = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;

    if !res.status.success() {
        return None;
    }

    let res = String::from_utf8(res.stdout).ok()?;

    Some(res)
}

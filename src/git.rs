use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub struct Repo {
    pub root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ChangedFile {
    pub status: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct GitState {
    pub branch: Option<String>,
    pub commit: Option<String>,
    pub working_tree: String,
    pub status_short: String,
    pub changed_files: Vec<ChangedFile>,
    pub diff_stat: String,
    pub recent_commits: String,
    pub full_diff: Option<String>,
}

pub fn detect_repo() -> Result<Repo, String> {
    let output = git(&["rev-parse", "--show-toplevel"], None)?;
    let root = output.trim();
    if root.is_empty() {
        return Err("Handoff must be run inside a git repository.\n\nTip: cd into your project and try again.".into());
    }
    Ok(Repo {
        root: PathBuf::from(root),
    })
}

impl GitState {
    pub fn capture(repo: &Path, include_diff: bool) -> Result<Self, String> {
        let branch =
            empty_to_none(git(&["branch", "--show-current"], Some(repo)).unwrap_or_default());
        let commit =
            empty_to_none(git(&["rev-parse", "--short", "HEAD"], Some(repo)).unwrap_or_default());
        let status_short = git(&["status", "--short"], Some(repo)).unwrap_or_default();
        let working_tree = if status_short.trim().is_empty() {
            "clean".to_string()
        } else {
            "dirty".to_string()
        };
        let mut changed_files =
            parse_changed_files(&git(&["diff", "--name-status"], Some(repo)).unwrap_or_default());
        for file in parse_status_files(&status_short) {
            if !changed_files.iter().any(|known| known.path == file.path) {
                changed_files.push(file);
            }
        }
        let diff_stat = git(&["diff", "--stat"], Some(repo)).unwrap_or_default();
        let recent_commits = git(&["log", "--oneline", "-5"], Some(repo)).unwrap_or_default();
        let full_diff = include_diff.then(|| git(&["diff"], Some(repo)).unwrap_or_default());

        Ok(Self {
            branch,
            commit,
            working_tree,
            status_short,
            changed_files,
            diff_stat,
            recent_commits,
            full_diff,
        })
    }
}

fn git(args: &[&str], cwd: Option<&Path>) -> Result<String, String> {
    let mut command = Command::new("git");
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let output = command
        .output()
        .map_err(|error| format!("failed to run git {}: {error}", args.join(" ")))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string())
}

fn empty_to_none(value: String) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

pub fn parse_changed_files(input: &str) -> Vec<ChangedFile> {
    input
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let status = parts.next()?.to_string();
            let path = parts.collect::<Vec<_>>().join(" ");
            (!path.is_empty()).then_some(ChangedFile { status, path })
        })
        .collect()
}

pub fn parse_status_files(input: &str) -> Vec<ChangedFile> {
    input
        .lines()
        .filter_map(|line| {
            if line.len() < 4 {
                return None;
            }
            let status = line[..2].trim().to_string();
            let path = line[3..].trim().to_string();
            (!path.is_empty()).then_some(ChangedFile { status, path })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_name_status_output() {
        let files = parse_changed_files("M\tsrc/main.rs\nA\tREADME.md\n");
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].status, "M");
        assert_eq!(files[0].path, "src/main.rs");
    }

    #[test]
    fn parses_status_short_output() {
        let files = parse_status_files("?? Cargo.toml\n M src/main.rs\n");
        assert_eq!(files[0].status, "??");
        assert_eq!(files[0].path, "Cargo.toml");
        assert_eq!(files[1].status, "M");
        assert_eq!(files[1].path, "src/main.rs");
    }
}

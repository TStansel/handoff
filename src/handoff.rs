use crate::agents::AgentName;
use crate::git::GitState;
use crate::transcript::TranscriptContext;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct HandoffPacket {
    pub source: AgentName,
    pub target: Option<AgentName>,
    pub created: String,
    pub repo: PathBuf,
    pub include_repo: bool,
    pub branch: Option<String>,
    pub commit: Option<String>,
    pub working_tree: Option<String>,
    pub handoff_file: PathBuf,
    pub source_session: Option<PathBuf>,
    pub files_touched: Vec<String>,
    pub decisions: Vec<String>,
    pub commands_run: Vec<String>,
    pub tests_checks: Vec<String>,
    pub known_issues: Vec<String>,
    pub open_questions: Vec<String>,
    pub next_steps: Vec<String>,
    pub suggested_prompt: String,
    pub raw_notes: Vec<String>,
    pub diff_stat: Option<String>,
    pub status_short: Option<String>,
    pub recent_commits: Option<String>,
    pub full_diff: Option<String>,
}

pub struct HandoffOptions {
    pub output: PathBuf,
    pub include_history: bool,
    pub include_raw: bool,
    pub source_session: Option<PathBuf>,
}

pub struct WrittenHandoff {
    pub latest: PathBuf,
    pub history: Option<PathBuf>,
    pub raw: Option<PathBuf>,
}

pub fn build_packet(
    source: AgentName,
    target: Option<AgentName>,
    repo: &Path,
    git: Option<&GitState>,
    source_session: Option<&Path>,
    transcript: &TranscriptContext,
    output: &Path,
) -> HandoffPacket {
    let include_repo = git.is_some();
    let mut files = git
        .map(|git| {
            git.changed_files
                .iter()
                .map(|file| format!("{} {}", file.status, file.path))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for file in &transcript.files {
        if !files.iter().any(|known| known.contains(file)) {
            files.push(file.clone());
        }
    }
    if files.is_empty() {
        files.push("No changed files detected from the transcript.".into());
    }

    let mut commands = transcript.commands.clone();
    if include_repo {
        for command in [
            "git rev-parse --show-toplevel",
            "git branch --show-current",
            "git rev-parse --short HEAD",
            "git status --short",
            "git diff --stat",
            "git diff --name-status",
            "git log --oneline -5",
        ] {
            if !commands.iter().any(|known| known == command) {
                commands.push(command.into());
            }
        }
    }

    let handoff_ref = display_relative(output, repo);
    let mut next_steps = vec![format!("Inspect `{handoff_ref}` before editing.")];
    if include_repo {
        next_steps
            .push("Verify repo state with `git status --short` and `git diff --stat`.".into());
    }
    next_steps.push("Review the files listed in Files Touched before editing.".into());
    next_steps.push(
        "Continue from the current goal and verify changes with the smallest relevant test or check."
            .into(),
    );
    let source_name = source.as_str();
    let suggested_prompt = if include_repo {
        format!("Read {handoff_ref} and continue from the next recommended step. Before editing, inspect the listed files and verify repo state.")
    } else {
        format!("Read {handoff_ref} and continue from the next recommended step. Before editing, inspect the listed files.")
    };

    HandoffPacket {
        source,
        target,
        created: now_string(),
        repo: repo.to_path_buf(),
        include_repo,
        branch: git.and_then(|git| git.branch.clone()),
        commit: git.and_then(|git| git.commit.clone()),
        working_tree: git.map(|git| git.working_tree.clone()),
        handoff_file: output.to_path_buf(),
        source_session: source_session.map(Path::to_path_buf),
        files_touched: files,
        decisions: vec![
            "Use `.agent-handoff/latest.md` as the durable cross-agent context file.".into(),
            if include_repo {
                "Repo state was included because `--repo` was passed.".into()
            } else {
                "Repo state was omitted because `--repo` was not passed.".into()
            },
        ],
        commands_run: commands,
        tests_checks: detect_tests(&transcript.commands),
        known_issues: non_empty_or_default(transcript.issues.clone(), "No known issues detected from the transcript."),
        open_questions: non_empty_or_default(transcript.questions.clone(), "No open questions detected from the transcript."),
        next_steps,
        suggested_prompt,
        raw_notes: non_empty_or_default(
            transcript.messages.clone(),
            &format!("No readable {source_name} transcript notes were available. Use repo state as the source of truth."),
        ),
        diff_stat: git.map(|git| git.diff_stat.clone()),
        status_short: git.map(|git| git.status_short.clone()),
        recent_commits: git.map(|git| git.recent_commits.clone()),
        full_diff: git.and_then(|git| git.full_diff.clone()),
    }
}

pub fn render_markdown(packet: &HandoffPacket) -> String {
    let target = packet.target.map(|a| a.as_str()).unwrap_or("unspecified");
    let mut out = String::new();
    out.push_str("# Handoff Packet\n\n");
    out.push_str("## Metadata\n\n");
    bullet(&mut out, "Source agent", packet.source.as_str());
    bullet(&mut out, "Target agent", target);
    bullet(&mut out, "Created", &packet.created);
    if packet.include_repo {
        bullet(&mut out, "Repo", &packet.repo.display().to_string());
        bullet(
            &mut out,
            "Branch",
            packet.branch.as_deref().unwrap_or("unknown"),
        );
        bullet(
            &mut out,
            "Commit",
            packet.commit.as_deref().unwrap_or("unknown"),
        );
        bullet(
            &mut out,
            "Working tree",
            packet.working_tree.as_deref().unwrap_or("unknown"),
        );
    }
    bullet(
        &mut out,
        "Handoff file",
        &display_relative(&packet.handoff_file, &packet.repo),
    );
    if let Some(session) = &packet.source_session {
        bullet(&mut out, "Source session", &session.display().to_string());
    }
    out.push('\n');

    section_text(
        &mut out,
        "Current Goal",
        "Continue the active coding task from the prior agent session using this handoff packet.",
    );
    section_text(&mut out, "Current State", &current_state(packet));
    section_list(&mut out, "Files Touched", &packet.files_touched);
    section_list(&mut out, "Important Decisions", &packet.decisions);
    section_list(&mut out, "Commands Run", &packet.commands_run);
    section_list(&mut out, "Tests / Checks", &packet.tests_checks);
    section_list(&mut out, "Known Issues", &packet.known_issues);
    section_list(&mut out, "Open Questions", &packet.open_questions);
    section_numbered(&mut out, "Next Recommended Steps", &packet.next_steps);
    section_text(
        &mut out,
        "Suggested Prompt for Target Agent",
        &packet.suggested_prompt,
    );
    section_list(&mut out, "Raw Context Notes", &packet.raw_notes);

    if packet.include_repo {
        out.push_str("## Git Snapshot\n\n");
        out.push_str("### Status\n\n```text\n");
        let status_short = packet.status_short.as_deref().unwrap_or_default();
        out.push_str(if status_short.trim().is_empty() {
            "clean\n"
        } else {
            status_short
        });
        out.push_str("\n```\n\n### Diff Stat\n\n```text\n");
        let diff_stat = packet.diff_stat.as_deref().unwrap_or_default();
        out.push_str(if diff_stat.trim().is_empty() {
            "No unstaged diff.\n"
        } else {
            diff_stat
        });
        out.push_str("\n```\n\n### Recent Commits\n\n```text\n");
        let recent_commits = packet.recent_commits.as_deref().unwrap_or_default();
        out.push_str(if recent_commits.trim().is_empty() {
            "No commits detected.\n"
        } else {
            recent_commits
        });
        out.push_str("\n```\n");
    }

    if let Some(diff) = &packet.full_diff {
        out.push_str("\n## Full Diff\n\n```diff\n");
        out.push_str(diff);
        out.push_str("\n```\n");
    }

    out
}

pub fn write_handoff(
    repo: &Path,
    source: AgentName,
    target: Option<AgentName>,
    markdown: &str,
    options: &HandoffOptions,
) -> Result<WrittenHandoff, String> {
    let latest = if options.output.is_absolute() {
        options.output.clone()
    } else {
        repo.join(&options.output)
    };
    if let Some(parent) = latest.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }
    fs::write(&latest, markdown)
        .map_err(|error| format!("failed to write {}: {error}", latest.display()))?;

    let history = if options.include_history {
        let history_dir = repo.join(".agent-handoff/history");
        fs::create_dir_all(&history_dir)
            .map_err(|error| format!("failed to create {}: {error}", history_dir.display()))?;
        let history_name = format!(
            "{}-{}-to-{}.md",
            timestamp_slug(),
            source.as_str(),
            target.map(|a| a.as_str()).unwrap_or("unspecified")
        );
        let history = history_dir.join(history_name);
        fs::write(&history, markdown)
            .map_err(|error| format!("failed to write {}: {error}", history.display()))?;
        Some(history)
    } else {
        None
    };

    let raw = if options.include_raw {
        if let Some(session) = &options.source_session {
            let raw_dir = repo.join(".agent-handoff/raw");
            fs::create_dir_all(&raw_dir)
                .map_err(|error| format!("failed to create {}: {error}", raw_dir.display()))?;
            let name = session.file_name().unwrap_or_default();
            let destination = raw_dir.join(name);
            fs::copy(session, &destination).map_err(|error| {
                format!("failed to copy raw session {}: {error}", session.display())
            })?;
            Some(destination)
        } else {
            None
        }
    } else {
        None
    };

    Ok(WrittenHandoff {
        latest,
        history,
        raw,
    })
}

fn current_state(packet: &HandoffPacket) -> String {
    if packet.include_repo {
        format!(
            "Git reports a {} working tree on branch `{}` at commit `{}`. Review the Git Snapshot below for changed files and recent commits.",
            packet.working_tree.as_deref().unwrap_or("unknown"),
            packet.branch.as_deref().unwrap_or("unknown"),
            packet.commit.as_deref().unwrap_or("unknown")
        )
    } else {
        "Repo state was not included in this packet. Re-run with `--repo` to include branch, commit, working tree, changed files, and Git snapshot details.".into()
    }
}

fn detect_tests(commands: &[String]) -> Vec<String> {
    let tests = commands
        .iter()
        .filter(|command| {
            let lower = command.to_ascii_lowercase();
            lower.contains("test")
                || lower.contains("lint")
                || lower.contains("check")
                || lower.contains("build")
        })
        .cloned()
        .collect::<Vec<_>>();
    non_empty_or_default(tests, "No tests or checks detected from the transcript.")
}

fn non_empty_or_default(mut values: Vec<String>, default: &str) -> Vec<String> {
    values.retain(|value| !value.trim().is_empty());
    if values.is_empty() {
        vec![default.to_string()]
    } else {
        values
    }
}

fn bullet(out: &mut String, label: &str, value: &str) {
    out.push_str(&format!("- {label}: {value}\n"));
}

fn section_text(out: &mut String, title: &str, text: &str) {
    out.push_str(&format!("## {title}\n\n{text}\n\n"));
}

fn section_list(out: &mut String, title: &str, values: &[String]) {
    out.push_str(&format!("## {title}\n\n"));
    for value in values {
        out.push_str(&format!("- {value}\n"));
    }
    out.push('\n');
}

fn section_numbered(out: &mut String, title: &str, values: &[String]) {
    out.push_str(&format!("## {title}\n\n"));
    for (index, value) in values.iter().enumerate() {
        out.push_str(&format!("{}. {value}\n", index + 1));
    }
    out.push('\n');
}

fn display_relative(path: &Path, repo: &Path) -> String {
    path.strip_prefix(repo)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn now_string() -> String {
    Command::new("date")
        .arg("-Iseconds")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| format!("unix:{}", unix_now()))
}

fn timestamp_slug() -> String {
    unix_now().to_string()
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::GitState;

    #[test]
    fn renders_required_sections() {
        let git = GitState {
            branch: Some("main".into()),
            commit: Some("abc123".into()),
            working_tree: "clean".into(),
            status_short: String::new(),
            changed_files: vec![],
            diff_stat: String::new(),
            recent_commits: "abc123 init".into(),
            full_diff: None,
        };
        let packet = build_packet(
            AgentName::Codex,
            Some(AgentName::Claude),
            Path::new("/tmp/repo"),
            Some(&git),
            None,
            &TranscriptContext::default(),
            Path::new("/tmp/repo/.agent-handoff/latest.md"),
        );
        let rendered = render_markdown(&packet);
        assert!(rendered.contains("# Handoff Packet"));
        assert!(rendered.contains("## Metadata"));
        assert!(rendered.contains("## Next Recommended Steps"));
        assert!(rendered.contains("## Suggested Prompt for Target Agent"));
    }
}

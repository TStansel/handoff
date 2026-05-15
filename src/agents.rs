use crate::paths::home_dir;
use crate::transcript::{
    extract_json_string_values_for_keys, parse_transcript_file, TranscriptContext,
};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentName {
    Codex,
    Claude,
}

impl AgentName {
    pub fn as_str(self) -> &'static str {
        match self {
            AgentName::Codex => "codex",
            AgentName::Claude => "claude",
        }
    }

    pub fn command_name(self) -> &'static str {
        match self {
            AgentName::Codex => "codex",
            AgentName::Claude => "claude",
        }
    }
}

pub fn agent_display_name(agent: AgentName) -> &'static str {
    match agent {
        AgentName::Codex => "Codex",
        AgentName::Claude => "Claude",
    }
}

#[derive(Debug, Clone)]
pub struct DetectedSession {
    pub agent: AgentName,
    pub path: PathBuf,
    pub modified: u64,
    repo_score: u8,
}

impl DetectedSession {
    pub fn matches_session_ref(&self, session_ref: &str) -> bool {
        let file_name = self.path.file_name().and_then(|name| name.to_str());
        let stem = self.path.file_stem().and_then(|name| name.to_str());
        file_name.is_some_and(|name| name == session_ref || name.contains(session_ref))
            || stem.is_some_and(|name| name == session_ref || name.ends_with(session_ref))
    }
}

pub fn detect_sessions(agent: AgentName, repo_path: &Path) -> Result<Vec<DetectedSession>, String> {
    let Some(home) = home_dir() else {
        return Ok(vec![]);
    };
    let roots = match agent {
        AgentName::Codex => vec![home.join(".codex").join("sessions")],
        AgentName::Claude => claude_search_roots(&home, repo_path),
    };

    let mut sessions = Vec::new();
    for root in roots {
        if !root.exists() {
            continue;
        }
        visit(&root, 0, &mut |path| {
            if looks_like_session(agent, path) {
                if let Some(session) = detected_session(agent, path, repo_path) {
                    sessions.push(session);
                }
            }
        })?;
    }

    sessions.sort_by(|a, b| {
        b.repo_score
            .cmp(&a.repo_score)
            .then_with(|| b.modified.cmp(&a.modified))
    });
    sessions.dedup_by(|a, b| a.path == b.path);
    Ok(sessions)
}

pub fn read_session_context(agent: AgentName, path: &Path) -> Result<TranscriptContext, String> {
    parse_transcript_file(agent, path)
}

fn visit(dir: &Path, depth: usize, on_file: &mut impl FnMut(&Path)) -> Result<(), String> {
    if depth > 6 {
        return Ok(());
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit(&path, depth + 1, on_file)?;
        } else {
            on_file(&path);
        }
    }
    Ok(())
}

fn looks_like_session(agent: AgentName, path: &Path) -> bool {
    match agent {
        AgentName::Codex => path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("rollout-") && name.ends_with(".jsonl")),
        AgentName::Claude => matches!(path.extension().and_then(|ext| ext.to_str()), Some("jsonl")),
    }
}

fn detected_session(agent: AgentName, path: &Path, repo_path: &Path) -> Option<DetectedSession> {
    let metadata = fs::metadata(path).ok()?;
    let modified = metadata
        .modified()
        .unwrap_or(SystemTime::UNIX_EPOCH)
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    Some(DetectedSession {
        agent,
        path: path.to_path_buf(),
        modified,
        repo_score: score_repo_match(agent, path, repo_path),
    })
}

fn score_repo_match(agent: AgentName, session_path: &Path, repo_path: &Path) -> u8 {
    if session_cwd_matches(session_path, repo_path) {
        return 4;
    }
    let file_name = session_path.to_string_lossy();
    let repo = repo_path.to_string_lossy();
    let basename = repo_path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    if agent == AgentName::Claude
        && file_name.contains(
            &encode_claude_project_key(repo_path)
                .to_string_lossy()
                .to_string(),
        )
    {
        3
    } else if file_name.contains(repo.as_ref()) {
        2
    } else if !basename.is_empty() && file_name.contains(basename) {
        1
    } else {
        0
    }
}

fn session_cwd_matches(session_path: &Path, repo_path: &Path) -> bool {
    let Ok(file) = fs::File::open(session_path) else {
        return false;
    };
    let repo = normalize_path(repo_path);
    BufReader::new(file).lines().take(40).flatten().any(|line| {
        extract_json_string_values_for_keys(&line, &["cwd"])
            .iter()
            .any(|cwd| normalize_path(Path::new(cwd)) == repo)
    })
}

fn claude_search_roots(home: &Path, repo_path: &Path) -> Vec<PathBuf> {
    let projects = home.join(".claude").join("projects");
    vec![
        projects.join(encode_claude_project_key(repo_path)),
        projects,
    ]
}

fn encode_claude_project_key(path: &Path) -> PathBuf {
    let raw = path.to_string_lossy();
    let encoded = raw
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '.' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>();
    PathBuf::from(encoded)
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy()
        .trim_end_matches('/')
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_claude_project_key_like_project_path() {
        assert_eq!(
            encode_claude_project_key(Path::new("/Users/thomas/projects/handoff")),
            PathBuf::from("-Users-thomas-projects-handoff")
        );
    }

    #[test]
    fn matches_codex_rollout_by_uuid() {
        let session = DetectedSession {
            agent: AgentName::Codex,
            path: PathBuf::from(
                "/tmp/rollout-2026-05-14T20-11-13-019e2966-f348-7252-9169-8a7ef9f580fe.jsonl",
            ),
            modified: 0,
            repo_score: 0,
        };
        assert!(session.matches_session_ref("019e2966-f348-7252-9169-8a7ef9f580fe"));
    }

    #[test]
    fn matches_claude_session_by_stem() {
        let session = DetectedSession {
            agent: AgentName::Claude,
            path: PathBuf::from("/tmp/019e2966-f348-7252-9169-8a7ef9f580fe.jsonl"),
            modified: 0,
            repo_score: 0,
        };
        assert!(session.matches_session_ref("019e2966-f348-7252-9169-8a7ef9f580fe"));
    }
}

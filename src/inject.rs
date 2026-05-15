use crate::agents::AgentName;
use std::fs;
use std::path::{Path, PathBuf};

const START: &str = "<!-- HANDOFF:START -->";
const END: &str = "<!-- HANDOFF:END -->";

pub fn inject_for_agent(repo: &Path, agent: AgentName) -> Result<PathBuf, String> {
    let file_name = match agent {
        AgentName::Codex => "AGENTS.md",
        AgentName::Claude => "CLAUDE.md",
    };
    let path = repo.join(file_name);
    let existing = fs::read_to_string(&path).unwrap_or_default();
    let updated = inject_block(&existing);
    fs::write(&path, updated)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    Ok(path)
}

pub fn inject_block(existing: &str) -> String {
    let block = format!(
        "{START}\n## Handoff Context\n\nWhen starting or resuming work in this repository, read:\n\n- `.agent-handoff/latest.md`\n\nThis file contains the latest cross-agent task state, decisions, files touched, known issues, and next recommended steps.\n{END}"
    );

    if let (Some(start), Some(end)) = (existing.find(START), existing.find(END)) {
        let end_index = end + END.len();
        let mut updated = String::new();
        updated.push_str(existing[..start].trim_end());
        if !updated.is_empty() {
            updated.push_str("\n\n");
        }
        updated.push_str(&block);
        let suffix = existing[end_index..].trim_start();
        if !suffix.is_empty() {
            updated.push_str("\n\n");
            updated.push_str(suffix);
        }
        updated.push('\n');
        updated
    } else if existing.trim().is_empty() {
        format!("{block}\n")
    } else {
        format!("{}\n\n{block}\n", existing.trim_end())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_without_overwriting_user_content() {
        let updated = inject_block("# Notes\n\nKeep this.");
        assert!(updated.contains("# Notes"));
        assert!(updated.contains(START));
    }

    #[test]
    fn replaces_existing_marked_block_only() {
        let updated = inject_block("# A\n\n<!-- HANDOFF:START -->old<!-- HANDOFF:END -->\n\n# B\n");
        assert!(updated.contains("# A"));
        assert!(updated.contains("# B"));
        assert!(!updated.contains("old"));
        assert_eq!(updated.matches(START).count(), 1);
    }
}

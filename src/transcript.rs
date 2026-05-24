use crate::agents::AgentName;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct TranscriptContext {
    pub messages: Vec<String>,
    pub commands: Vec<String>,
    pub files: Vec<String>,
    pub issues: Vec<String>,
    pub questions: Vec<String>,
}

pub fn parse_transcript_file(agent: AgentName, path: &Path) -> Result<TranscriptContext, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("failed to read session {}: {error}", path.display()))?;
    Ok(parse_transcript(agent, &content))
}

pub fn parse_transcript(agent: AgentName, content: &str) -> TranscriptContext {
    let mut context = TranscriptContext::default();
    let mut files = BTreeSet::new();
    let mut commands = BTreeSet::new();

    for line in content.lines() {
        let texts = extract_string_values(agent, line);
        for text in texts {
            let normalized = text.replace("\\n", "\n").replace("\\\"", "\"");
            let trimmed = normalized.trim();
            if trimmed.len() < 3 {
                continue;
            }
            if trimmed.len() > 5000 {
                continue;
            }
            if looks_like_code_or_metadata(trimmed) {
                continue;
            }
            for command in detect_commands(trimmed) {
                commands.insert(command);
            }
            for file in detect_files(trimmed) {
                files.insert(file);
            }
            if looks_like_issue(trimmed) {
                push_limited(&mut context.issues, one_line(trimmed), 12);
            }
            if trimmed.ends_with('?') {
                push_limited(&mut context.questions, one_line(trimmed), 8);
            }
            if is_meaningful_message(trimmed) {
                push_limited(&mut context.messages, one_line(trimmed), 18);
            }
        }
    }

    context.commands = commands.into_iter().take(30).collect();
    context.files = files.into_iter().take(50).collect();
    context
}

fn extract_string_values(agent: AgentName, line: &str) -> Vec<String> {
    if !line.trim_start().starts_with('{') {
        return vec![line.to_string()];
    }
    match agent {
        AgentName::Codex => {
            if line.contains("\"type\":\"session_meta\"")
                || line.contains("\"type\":\"token_count\"")
                || line.contains("\"type\":\"turn_context\"")
            {
                return Vec::new();
            }
        }
        AgentName::Claude => {
            if line.contains("\"type\":\"summary\"") {
                return Vec::new();
            }
        }
        AgentName::CursorAgent => {}
    }
    if let Some(role) = extract_json_string_values_for_keys(line, &["role"]).first() {
        if !matches!(role.as_str(), "user" | "assistant") {
            return Vec::new();
        }
    }
    let preferred = extract_json_string_values_for_keys(
        line,
        &["content", "text", "message", "cmd", "command", "output"],
    );
    if !preferred.is_empty() {
        return preferred;
    }
    Vec::new()
}

pub fn extract_json_string_values_for_keys(line: &str, keys: &[&str]) -> Vec<String> {
    let mut values = Vec::new();
    for key in keys {
        let needle = format!("\"{key}\"");
        let mut rest = line;
        while let Some(index) = rest.find(&needle) {
            let start = index + needle.len();
            let after = &rest[start..];
            let Some(value_start) = json_string_value_start(after) else {
                rest = &after[1.min(after.len())..];
                continue;
            };
            let after_value_start = &after[value_start..];
            let (value, consumed) = read_json_string(after_value_start);
            if !value.trim().is_empty() {
                values.push(value);
            }
            let total_consumed = value_start + consumed;
            if total_consumed >= after.len() {
                break;
            }
            rest = &after[total_consumed..];
        }
    }
    values
}

fn json_string_value_start(input: &str) -> Option<usize> {
    let mut chars = input.char_indices().peekable();
    while let Some((_, ch)) = chars.peek().copied() {
        if ch.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
    if chars.next()?.1 != ':' {
        return None;
    }
    while let Some((_, ch)) = chars.peek().copied() {
        if ch.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
    let (index, ch) = chars.next()?;
    (ch == '"').then_some(index + 1)
}

fn read_json_string(input: &str) -> (String, usize) {
    let mut value = String::new();
    let mut escaped = false;
    for (index, ch) in input.char_indices() {
        if escaped {
            value.push(match ch {
                'n' => '\n',
                't' => '\t',
                other => other,
            });
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            return (value, index + 1);
        } else {
            value.push(ch);
        }
    }
    (value, input.len())
}

fn detect_commands(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim().trim_start_matches('$').trim();
        if starts_with_command(trimmed) {
            found.push(trimmed.to_string());
        }
    }
    found
}

fn starts_with_command(line: &str) -> bool {
    let commands = [
        "git ", "cargo ", "npm ", "pnpm ", "yarn ", "node ", "python ", "python3 ", "pytest ",
        "go ", "rustc ", "npx ", "make ", "cmake ", "rg ", "sed ", "cat ", "ls ",
    ];
    commands.iter().any(|prefix| line.starts_with(prefix))
}

fn detect_files(text: &str) -> Vec<String> {
    text.split(|ch: char| ch.is_whitespace() || matches!(ch, ',' | ')' | '(' | '[' | ']'))
        .map(|token| token.trim_matches(|ch: char| matches!(ch, '`' | '\'' | '"' | ':' | ';')))
        .map(|token| token.trim_end_matches(|ch: char| matches!(ch, '?' | '.' | ',')))
        .filter(|token| {
            !token.starts_with('<')
                && !token.contains("\\t")
                && !token.contains("::")
                && !token.ends_with('/')
                && (token.contains('/')
                    || token.ends_with(".rs")
                    || token.ends_with(".ts")
                    || token.ends_with(".tsx")
                    || token.ends_with(".js")
                    || token.ends_with(".json")
                    || token.ends_with(".md")
                    || token.ends_with(".toml"))
        })
        .filter(|token| !token.starts_with("http"))
        .map(ToString::to_string)
        .collect()
}

fn looks_like_issue(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    [
        "error", "failed", "failing", "bug", "panic", "todo", "missing", "blocker",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn looks_like_code_or_metadata(text: &str) -> bool {
    let trimmed = text.trim_start();
    if trimmed.starts_with("use ")
        || trimmed.starts_with("pub ")
        || trimmed.starts_with("fn ")
        || trimmed.starts_with("impl ")
        || trimmed.starts_with("mod ")
        || trimmed.starts_with("const ")
        || trimmed.starts_with("#[")
        || trimmed.starts_with("*** Begin Patch")
        || trimmed.starts_with("Chunk ID:")
    {
        return true;
    }
    let punctuation = text
        .chars()
        .filter(|ch| matches!(ch, '{' | '}' | ';'))
        .count();
    punctuation > 20
}

fn is_meaningful_message(text: &str) -> bool {
    text.len() > 20 && !starts_with_command(text) && !text.starts_with("tool_")
}

fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn push_limited(values: &mut Vec<String>, value: String, limit: usize) {
    if values.last() == Some(&value) {
        return;
    }
    values.push(value);
    if values.len() > limit {
        values.remove(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_commands_files_and_messages() {
        let parsed = parse_transcript(
            AgentName::Codex,
            r#"{"role":"user","content":"Run cargo test after editing src/main.rs?"}
{"role":"assistant","content":"cargo test\nIt failed in tests/render.rs"}"#,
        );
        assert!(parsed.commands.contains(&"cargo test".to_string()));
        assert!(parsed.files.iter().any(|file| file.contains("src/main.rs")));
        assert!(!parsed.messages.is_empty());
    }

    #[test]
    fn skips_codex_session_metadata() {
        let parsed = parse_transcript(
            AgentName::Codex,
            r#"{"timestamp":"now","type":"session_meta","payload":{"cwd":"/tmp/repo","base_instructions":{"text":"system prompt"}}}
{"type":"response_item","payload":{"role":"assistant","content":"Use src/main.rs for the change."}}"#,
        );
        assert!(!parsed
            .messages
            .iter()
            .any(|message| message.contains("system prompt")));
        assert!(parsed.files.iter().any(|file| file == "src/main.rs"));
    }

    #[test]
    fn parses_cursor_agent_text_parts() {
        let parsed = parse_transcript(
            AgentName::CursorAgent,
            r#"{"role":"user","message":{"content":[{"type":"text","text":"Please edit src/agents.rs and run cargo test?"}]}}
{"role":"assistant","message":{"content":[{"type":"text","text":"cargo test\nThe change is in src/agents.rs."}]}}"#,
        );
        assert!(parsed.commands.contains(&"cargo test".to_string()));
        assert!(parsed.files.iter().any(|file| file == "src/agents.rs"));
        assert!(parsed
            .questions
            .iter()
            .any(|question| question.contains("src/agents.rs")));
    }
}

use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn handoff() -> Command {
    Command::new(env!("CARGO_BIN_EXE_handoff"))
}

fn workspace(name: &str) -> (PathBuf, PathBuf) {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("handoff-test-{name}-{nonce}"));
    let home = root.join("home");
    let repo = root.join("repo");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&repo).unwrap();
    (home, repo)
}

fn run(args: &[&str], name: &str) -> Output {
    let (home, repo) = workspace(name);
    handoff()
        .args(args)
        .current_dir(repo)
        .env("HOME", home)
        .env("USERPROFILE", "")
        .env("HANDOFF_NO_UPDATE_CHECK", "1")
        .output()
        .unwrap()
}

#[cfg(unix)]
fn run_with_fake_brew(args: &[&str], name: &str, brew_output: &str) -> Output {
    let (home, repo) = workspace(name);
    let bin = home.join("bin");
    fs::create_dir_all(&bin).unwrap();

    write_executable(
        &bin.join("brew"),
        &format!("#!/bin/sh\nprintf '%s\\n' '{}'\n", brew_output),
    );
    write_executable(
        &bin.join("codex"),
        "#!/bin/sh\nprintf 'codex launched: %s\\n' \"$1\"\n",
    );
    write_executable(
        &bin.join("claude"),
        "#!/bin/sh\nprintf 'claude launched: %s\\n' \"$1\"\n",
    );

    let path = std::env::var_os("PATH").unwrap_or_default();
    handoff()
        .args(args)
        .current_dir(repo)
        .env("HOME", home)
        .env("USERPROFILE", "")
        .env(
            "PATH",
            format!("{}:{}", bin.display(), path.to_string_lossy()),
        )
        .output()
        .unwrap()
}

#[cfg(unix)]
fn run_with_fake_agents(args: &[&str], name: &str) -> Output {
    let (home, repo) = workspace(name);
    let bin = home.join("bin");
    fs::create_dir_all(&bin).unwrap();

    write_executable(
        &bin.join("codex"),
        "#!/bin/sh\nprintf 'codex launched: %s\\n' \"$1\"\n",
    );
    write_executable(
        &bin.join("claude"),
        "#!/bin/sh\nprintf 'claude launched: %s\\n' \"$1\"\n",
    );
    write_executable(
        &bin.join("gemini"),
        "#!/bin/sh\nprintf 'gemini launched: %s\\n' \"$1\"\n",
    );

    let path = std::env::var_os("PATH").unwrap_or_default();
    handoff()
        .args(args)
        .current_dir(repo)
        .env("HOME", home)
        .env("USERPROFILE", "")
        .env("HANDOFF_NO_UPDATE_CHECK", "1")
        .env(
            "PATH",
            format!("{}:{}", bin.display(), path.to_string_lossy()),
        )
        .output()
        .unwrap()
}

#[cfg(unix)]
fn write_executable(path: &std::path::Path, contents: &str) {
    fs::write(path, contents).unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

#[test]
fn help_and_version_succeed() {
    let help = run(&["--help"], "help");
    assert!(help.status.success(), "{}", stderr(&help));
    assert!(stdout(&help).contains("handoff status"));

    let version = run(&["--version"], "version");
    assert!(version.status.success(), "{}", stderr(&version));
    assert!(stdout(&version).contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn status_succeeds_without_git_or_sessions() {
    let output = run(&["status"], "status");
    assert!(output.status.success(), "{}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("Git repo: no"));
    assert!(text.contains("- codex: 0 candidate sessions"));
    assert!(text.contains("- claude: 0 candidate sessions"));
}

#[test]
fn inject_dry_run_covers_both_agents() {
    for agent in ["codex", "claude"] {
        let output = run(&["inject", agent, "--dry-run"], agent);
        assert!(output.status.success(), "{agent}: {}", stderr(&output));
        assert!(stdout(&output).contains("Would update"));
    }
}

#[test]
fn pull_dry_run_covers_both_sources() {
    for source in ["codex", "claude"] {
        let output = run(&["pull", source, "--dry-run"], source);
        assert!(output.status.success(), "{source}: {}", stderr(&output));
        assert!(stdout(&output).contains("# Handoff Packet"));
    }
}

#[test]
fn cross_agent_dry_run_covers_both_directions() {
    for args in [
        ["codex", "claude", "--dry-run"],
        ["claude", "codex", "--dry-run"],
    ] {
        let output = run(&args, args[0]);
        assert!(output.status.success(), "{args:?}: {}", stderr(&output));
        assert!(stdout(&output).contains("# Handoff Packet"));
    }
}

#[test]
fn source_only_handoff_prints_generic_prompt() {
    let output = run(&["claude"], "source-only");
    assert!(output.status.success(), "{}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("Created handoff:"));
    assert!(text.contains("Start your agent with this prompt:"));
    assert!(text.contains("Read .agent-handoff/latest.md"));
}

#[cfg(unix)]
#[test]
fn cross_agent_handoff_launches_target_agent() {
    let output = run_with_fake_agents(&["codex", "claude"], "auto-launch");
    assert!(output.status.success(), "{}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("Created handoff:"));
    assert!(text.contains("Starting Claude with:"));
    assert!(text.contains("claude \"Read .agent-handoff/latest.md"));
    assert!(text.contains("claude launched: Read .agent-handoff/latest.md"));
}

#[cfg(unix)]
#[test]
fn handoff_launches_arbitrary_target_agent() {
    let output = run_with_fake_agents(&["claude", "gemini"], "arbitrary-launch");
    assert!(output.status.success(), "{}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("Created handoff:"));
    assert!(text.contains("Starting gemini with:"));
    assert!(text.contains("gemini \"Read .agent-handoff/latest.md"));
    assert!(text.contains("gemini launched: Read .agent-handoff/latest.md"));
}

#[test]
fn expected_errors_are_reported() {
    let cases = [
        (vec!["unknown"], "unknown command"),
        (vec!["pull"], "usage: handoff pull"),
        (vec!["inject"], "usage: handoff inject"),
        (
            vec!["codex", "codex"],
            "source and target agents must differ",
        ),
        (
            vec!["pull", "codex", "--include-diff"],
            "--include-diff requires --repo",
        ),
        (vec!["pull", "codex", "--repo"], "--repo requires"),
    ];

    for (index, (args, expected)) in cases.into_iter().enumerate() {
        let output = run(&args, &format!("error-{index}"));
        assert!(!output.status.success(), "{args:?} unexpectedly succeeded");
        assert!(
            stderr(&output).contains(expected),
            "{args:?}: expected {expected:?}, got {:?}",
            stderr(&output)
        );
    }
}

#[cfg(unix)]
#[test]
fn cross_agent_handoff_prints_brew_update_notice_when_outdated() {
    let output = run_with_fake_brew(&["codex", "claude"], "update-notice", "handoff");
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stderr(&output).contains("brew update && brew upgrade handoff"));
}

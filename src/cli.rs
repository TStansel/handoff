use crate::agents::{
    agent_display_name, detect_sessions, read_session_context, AgentName, SUPPORTED_AGENT_CLIS,
};
use crate::git::{detect_workspace, GitState};
use crate::handoff::{build_packet, render_markdown, write_handoff, HandoffOptions};
use crate::inject::inject_for_agent;
use crate::update::maybe_print_update_notice;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Debug, Default)]
struct Flags {
    dry_run: bool,
    include_raw: bool,
    include_diff: bool,
    inject: bool,
    history: bool,
    repo: bool,
    verbose: bool,
    session: Option<PathBuf>,
    output: Option<PathBuf>,
}

pub fn run(args: Vec<String>) -> Result<(), String> {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        print_help();
        return Ok(());
    }
    if args[0] == "--version" || args[0] == "-V" {
        println!("handoff {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    match args[0].as_str() {
        "status" => status(parse_flags(&args[1..])?),
        "pull" => {
            if args.len() < 2 {
                return Err(format!("usage: handoff pull <{}>", supported_agent_names()));
            }
            let source = parse_source_agent(&args[1])?;
            pull(source, None, parse_flags(&args[2..])?, false)
        }
        "inject" => {
            if args.len() < 2 {
                return Err(format!(
                    "usage: handoff inject <{}>",
                    supported_agent_names()
                ));
            }
            let target = parse_source_agent(&args[1])?;
            inject(target, parse_flags(&args[2..])?)
        }
        source if parse_source_agent(source).is_ok() => {
            let source = parse_source_agent(source)?;
            let (target, flag_start) = match args.get(1) {
                Some(value) if !value.starts_with("--") => {
                    let target = parse_target_agent(value)?;
                    if source.as_str() == target {
                        return Err("source and target agents must differ".into());
                    }
                    (Some(target), 2)
                }
                _ => (None, 1),
            };
            pull(source, target, parse_flags(&args[flag_start..])?, true)
        }
        command => Err(format!(
            "unknown command `{command}`. Run `handoff --help`."
        )),
    }
}

fn print_help() {
    let supported = supported_agent_names();
    println!(
        "Handoff - hand off local coding-agent context\n\n\
Usage:\n  handoff status [--verbose]\n  handoff pull <{supported}> [options]\n  handoff <{supported}> [options]\n  handoff <{supported}> <target-agent> [options]\n  handoff inject <{supported}> [--dry-run]\n\n\
Options:\n  --dry-run           Print actions without writing files\n  --session <id|path> Use a specific source session id or file\n  --repo              Include git repo state in the handoff markdown\n  --include-raw       Copy raw source transcript into .agent-handoff/raw/\n  --include-diff      Include full git diff; requires --repo\n  --inject            Update the target agent context file with a handoff pointer\n  --history           Write a timestamped archive copy under .agent-handoff/history/\n  --output <path>     Write handoff to a custom path\n  --verbose           Show detection details\n  -h, --help          Show help\n  -V, --version       Show version"
    );
}

fn parse_flags(args: &[String]) -> Result<Flags, String> {
    let mut flags = Flags::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dry-run" => flags.dry_run = true,
            "--include-raw" => flags.include_raw = true,
            "--include-diff" => flags.include_diff = true,
            "--inject" => flags.inject = true,
            "--history" => flags.history = true,
            "--no-inject" => flags.inject = false,
            "--repo" => flags.repo = true,
            "--verbose" => flags.verbose = true,
            "--session" => {
                i += 1;
                flags.session = Some(PathBuf::from(
                    args.get(i).ok_or("--session requires a path")?,
                ));
            }
            "--output" => {
                i += 1;
                flags.output = Some(PathBuf::from(
                    args.get(i).ok_or("--output requires a path")?,
                ));
            }
            unknown => return Err(format!("unknown option `{unknown}`")),
        }
        i += 1;
    }
    Ok(flags)
}

fn parse_source_agent(value: &str) -> Result<AgentName, String> {
    for agent in SUPPORTED_AGENT_CLIS {
        if agent.as_str() == value {
            return Ok(agent);
        }
    }
    Err(format!(
        "unknown agent `{value}`; expected {}",
        supported_agent_names()
    ))
}

fn parse_target_agent(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.starts_with('-') {
        return Err("target agent must be a command name".into());
    }
    Ok(value.into())
}

fn parse_known_agent(value: &str) -> Option<AgentName> {
    parse_source_agent(value).ok()
}

fn display_target_agent(value: &str) -> String {
    parse_known_agent(value)
        .map(agent_display_name)
        .map(str::to_string)
        .unwrap_or_else(|| value.to_string())
}

fn run_agent(command_name: &str, prompt: &str) -> Result<(), String> {
    let status = Command::new(command_name)
        .arg(prompt)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| format!("failed to start {command_name}: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("{command_name} exited with status {status}"))
    }
}

fn supported_agent_names() -> String {
    SUPPORTED_AGENT_CLIS
        .iter()
        .map(|agent| agent.as_str())
        .collect::<Vec<_>>()
        .join("|")
}

fn status(flags: Flags) -> Result<(), String> {
    let workspace = detect_workspace()?;
    let git = workspace
        .git_root
        .as_ref()
        .and_then(|root| GitState::capture(root, false).ok());
    let detected = SUPPORTED_AGENT_CLIS
        .iter()
        .map(|agent| detect_sessions(*agent, &workspace.root).map(|sessions| (*agent, sessions)))
        .collect::<Result<Vec<_>, _>>()?;
    let latest = detected
        .iter()
        .flat_map(|(_, sessions)| sessions.iter())
        .max_by_key(|s| s.modified);
    let latest_handoff = workspace.root.join(".agent-handoff/latest.md");

    println!("Workspace: {}", workspace.root.display());
    match &git {
        Some(git) => {
            println!("Git repo: yes");
            println!("Git branch: {}", git.branch.as_deref().unwrap_or("unknown"));
            println!("Working tree: {}", git.working_tree);
        }
        None => println!("Git repo: no"),
    }
    println!();
    println!("Detected agents:");
    for (agent, sessions) in &detected {
        println!(
            "- {}: {} candidate sessions",
            agent.as_str(),
            sessions.len()
        );
    }
    if let Some(session) = latest {
        println!();
        println!("Latest likely source:");
        println!("- {}", session.agent.as_str());
        println!("- modified: {}", session.modified);
        println!("- session file: {}", session.path.display());
    }
    println!();
    println!("Handoff:");
    println!("- latest: .agent-handoff/latest.md");
    println!(
        "- exists: {}",
        if latest_handoff.exists() { "yes" } else { "no" }
    );

    if flags.verbose {
        for (agent, sessions) in &detected {
            print_sessions(agent.as_str(), sessions);
        }
    }

    Ok(())
}

fn print_sessions(label: &str, sessions: &[crate::agents::DetectedSession]) {
    println!();
    println!("{label} candidates:");
    if sessions.is_empty() {
        println!("- none");
    }
    for session in sessions.iter().take(10) {
        println!("- {} ({})", session.path.display(), session.modified);
    }
}

fn inject(target: AgentName, flags: Flags) -> Result<(), String> {
    let workspace = detect_workspace()?;
    let target_file = match target {
        AgentName::Codex => "AGENTS.md",
        AgentName::Claude => "CLAUDE.md",
        AgentName::CursorAgent => "AGENTS.md",
    };
    if flags.dry_run {
        println!(
            "Would update {}",
            workspace.root.join(target_file).display()
        );
        return Ok(());
    }
    let path = inject_for_agent(&workspace.root, target)?;
    println!("Updated:\n{}", path.display());
    Ok(())
}

fn pull(
    source: AgentName,
    target: Option<String>,
    flags: Flags,
    print_generic_prompt: bool,
) -> Result<(), String> {
    if flags.include_diff && !flags.repo {
        return Err(
            "--include-diff requires --repo because full diffs are repository context".into(),
        );
    }
    let workspace = detect_workspace()?;
    if flags.repo && workspace.git_root.is_none() {
        return Err("--repo requires running inside a git repository".into());
    }
    let git = if flags.repo {
        Some(GitState::capture(
            workspace.git_root.as_deref().unwrap(),
            flags.include_diff,
        )?)
    } else {
        None
    };
    let session_path = select_session(
        source,
        &workspace.root,
        flags.session.clone(),
        flags.verbose,
    )?;
    let transcript = match &session_path {
        Some(path) => read_session_context(source, path)?,
        None => Default::default(),
    };
    let output = flags
        .output
        .clone()
        .unwrap_or_else(|| workspace.root.join(".agent-handoff/latest.md"));

    let packet = build_packet(
        source,
        target.as_deref(),
        &workspace.root,
        git.as_ref(),
        session_path.as_deref(),
        &transcript,
        &output,
    );
    let markdown = render_markdown(&packet);

    if flags.dry_run {
        println!("{markdown}");
        return Ok(());
    }

    let write_options = HandoffOptions {
        output,
        include_history: flags.history,
        include_raw: flags.include_raw,
        source_session: session_path.clone(),
    };
    let written = write_handoff(
        &workspace.root,
        source,
        target.as_deref(),
        &markdown,
        &write_options,
    )?;

    println!("Created handoff:\n{}", written.latest.display());
    if let Some(history) = written.history {
        println!("\nArchived:\n{}", history.display());
    }
    if let Some(raw) = written.raw {
        println!("\nCopied raw session:\n{}", raw.display());
    }

    if let Some(target_agent) = target {
        if flags.inject {
            let Some(known_target) = parse_known_agent(&target_agent) else {
                return Err(format!(
                    "--inject supports only these targets: {}",
                    supported_agent_names()
                ));
            };
            let injected = inject_for_agent(&workspace.root, known_target)?;
            println!("\nUpdated:\n{}", injected.display());
        }
        println!("\nStarting {} with:\n", display_target_agent(&target_agent));
        println!("{} \"{}\"", target_agent, packet.suggested_prompt);
        maybe_print_update_notice();
        run_agent(&target_agent, &packet.suggested_prompt)?;
    } else if print_generic_prompt {
        println!("\nStart your agent with this prompt:\n");
        println!("{}", packet.suggested_prompt);
    }

    Ok(())
}

fn select_session(
    source: AgentName,
    repo_root: &std::path::Path,
    explicit: Option<PathBuf>,
    verbose: bool,
) -> Result<Option<PathBuf>, String> {
    let sessions = detect_sessions(source, repo_root)?;

    if let Some(session_ref) = explicit {
        if session_ref.exists() {
            return Ok(Some(session_ref));
        }

        let session_ref_text = session_ref.to_string_lossy();
        let matches = sessions
            .iter()
            .filter(|session| session.matches_session_ref(&session_ref_text))
            .collect::<Vec<_>>();

        match matches.as_slice() {
            [session] => return Ok(Some(session.path.clone())),
            [] => {
                return Err(format!(
                    "session id or file was not found for {}: {}",
                    source.as_str(),
                    session_ref.display()
                ));
            }
            _ => {
                return Err(format!(
                    "session id matched multiple {} sessions: {}",
                    source.as_str(),
                    session_ref.display()
                ));
            }
        }
    }

    if verbose {
        println!("Detected {} {} sessions", sessions.len(), source.as_str());
    }
    Ok(sessions.first().map(|session| session.path.clone()))
}

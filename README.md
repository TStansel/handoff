# Handoff

Hit a coding-agent limit mid-refactor?

Handoff lets you hand off local coding context between agents like Codex and Claude Code.

```sh
handoff codex claude
```

It reads recent local session context when available, writes a markdown handoff file, and prints the command to start the next agent. Pass `--repo` when you also want Git branch, commit, working tree, changed files, and snapshot details included.

No server. No dashboard. No hidden state mutation. Just a local handoff file your next agent can read.

## Install

For local development:

```sh
cargo build
cargo run -- status
```

After installing the binary on your `PATH`:

```sh
handoff --help
```

## Demo

```sh
# You were working in Codex and want to move to Claude Code
handoff codex claude

# Handoff writes:
# .agent-handoff/latest.md
#
# Then start Claude:
claude "Read .agent-handoff/latest.md and continue from the next recommended step. Before editing, inspect the listed files."
```

Reverse direction:

```sh
handoff claude codex

codex "Read .agent-handoff/latest.md and continue from the next recommended step. Before editing, inspect the listed files."
```

## Commands

```sh
handoff status
handoff pull codex
handoff pull claude
handoff codex claude
handoff claude codex
handoff inject claude
handoff inject codex
```

Options:

```sh
--dry-run
--session <id|path>
--repo
--include-raw
--include-diff
--inject
--history
--output <path>
--verbose
```

By default, Handoff writes only the markdown output file. It does not update `CLAUDE.md` or `AGENTS.md`, does not create a history archive, and does not write `state.json`.

Add `--inject` if you want Handoff to add a marked pointer block to `CLAUDE.md` or `AGENTS.md`:

```sh
handoff codex claude --inject
```

Add `--history` if you want a timestamped archive copy:

```sh
handoff codex claude --history
```

By default, generated packets omit Git repo details. Add `--repo` to include repo state:

```sh
handoff codex claude --repo
```

Full diffs are also repo state, so `--include-diff` requires `--repo`:

```sh
handoff codex claude --repo --include-diff
```

Use `--session` when you want a specific session instead of the latest detected one. It accepts either a file path or a session id:

```sh
handoff codex claude --session 019e2966-f348-7252-9169-8a7ef9f580fe
handoff codex claude --session ~/.codex/sessions/2026/05/14/rollout-2026-05-14T20-11-13-019e2966-f348-7252-9169-8a7ef9f580fe.jsonl
```

## Privacy

Handoff is local-first. It reads local agent session files and writes markdown into the current repo. It does not upload transcript data.

Review `.agent-handoff/latest.md` before sharing a repo publicly. If you use `--include-raw`, `.agent-handoff/raw/` may contain sensitive transcript data.

Recommended `.gitignore` entry:

```gitignore
.agent-handoff/raw/
```

## Status

This is an MVP. It uses defensive heuristics for Codex and Claude Code session discovery because local session formats are private implementation details and may change.

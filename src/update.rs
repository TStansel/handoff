use std::process::{Command, Stdio};

pub fn maybe_print_update_notice() {
    if std::env::var_os("HANDOFF_NO_UPDATE_CHECK").is_some() {
        return;
    }

    let output = Command::new("brew")
        .args(["outdated", "--quiet", "handoff"])
        .env("HOMEBREW_NO_AUTO_UPDATE", "1")
        .stderr(Stdio::null())
        .output();

    let Ok(output) = output else {
        return;
    };
    if !output.status.success() {
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if has_handoff_update(&stdout) {
        eprintln!("\nUpdate available: run `brew update && brew upgrade handoff`.");
    }
}

fn has_handoff_update(output: &str) -> bool {
    output.lines().any(|line| line.trim() == "handoff")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_handoff_in_brew_outdated_output() {
        assert!(has_handoff_update("handoff\n"));
        assert!(has_handoff_update("other\nhandoff\n"));
        assert!(!has_handoff_update(""));
        assert!(!has_handoff_update("other\n"));
    }
}

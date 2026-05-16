mod agents;
mod cli;
mod git;
mod handoff;
mod inject;
mod paths;
mod transcript;
mod update;

use std::process::ExitCode;

fn main() -> ExitCode {
    match cli::run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}

//! Replays run logs written by the client (`%LOCALAPPDATA%\Ante\runs\*.jsonl`) and reports
//! the first step whose state differs, to retrace the game logic of a run.
//!
//! cargo run -p ante-core --example replay -- <log.jsonl>...
//! Add `--features extra-jokers` for logs of runs with the optional Jokers.

use std::process::ExitCode;

use ante_core::run_log::replay;

fn main() -> ExitCode {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    if paths.is_empty() {
        eprintln!("Usage: cargo run -p ante-core --example replay -- <log.jsonl>...");
        return ExitCode::FAILURE;
    }
    let mut failed = false;
    for path in paths {
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) => {
                eprintln!("{path}: {error}");
                failed = true;
                continue;
            }
        };
        match replay(text.lines()) {
            Ok(report) => match report.divergence {
                None => println!("{path}: {} steps replayed, no divergence", report.steps),
                Some(divergence) => {
                    failed = true;
                    println!(
                        "{path}: step {} diverges ({:?})\n  logged refusal:   {:?}\n  replayed refusal: {:?}\n  logged:   {:?}\n  replayed: {:?}",
                        divergence.index,
                        divergence.command,
                        divergence.logged_refusal,
                        divergence.replayed_refusal,
                        divergence.logged,
                        divergence.replayed,
                    );
                }
            },
            Err(error) => {
                failed = true;
                println!("{path}: {error}");
            }
        }
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

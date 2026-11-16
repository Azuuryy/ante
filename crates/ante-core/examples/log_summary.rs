//! Prints a run log as one line per command that matters, to read a run without
//! replaying it: the cards played or discarded, the state that changed, and the log's
//! explanations (blind, hand, effects, notes, Jokers, shop and pack). Selections are
//! folded into the next command.
//!
//! cargo run -p ante-core --example log_summary -- <log.jsonl>

use std::process::ExitCode;

use ante_core::run_log::{Checkpoint, CheckpointDelta, Explain, LogLine, LoggedCommand};

fn main() -> ExitCode {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("Usage: cargo run -p ante-core --example log_summary -- <log.jsonl>");
        return ExitCode::FAILURE;
    };
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("{path}: {error}");
            return ExitCode::FAILURE;
        }
    };
    let mut state = Checkpoint::default();
    for (number, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<LogLine>(line) {
            Ok(LogLine::Header(header)) => println!(
                "seed {} deck {} goal {} (format {}, Ante {})",
                header.seed, header.deck, header.goal, header.format_version, header.ante_version
            ),
            Ok(LogLine::Select(group)) => group.after.apply_to(&mut state),
            Ok(LogLine::Step(step)) if step.command.is_selection() && step.refused.is_none() => {
                step.after.apply_to(&mut state);
            }
            Ok(LogLine::Step(step)) => {
                let selected: Vec<&str> = state
                    .hand
                    .iter()
                    .filter_map(|card| card.strip_suffix('*'))
                    .map(|card| card.trim_end())
                    .collect();
                let mut out = format!("{:>5} {}", step.index, command_name(&step.command));
                if matches!(step.command, LoggedCommand::Play | LoggedCommand::Discard) {
                    out.push_str(&format!(" [{}]", selected.join(", ")));
                }
                if let Some(refusal) = &step.refused {
                    out.push_str(&format!(" REFUSED: {refusal}"));
                }
                let changes = changes(&state, &step.after);
                if !changes.is_empty() {
                    out.push_str(&format!(" | {}", changes.join(", ")));
                }
                println!("{out}");
                if let Some(explain) = &step.explain {
                    print_explain(explain);
                }
                step.after.apply_to(&mut state);
            }
            Err(error) => {
                eprintln!("line {}: {error}", number + 1);
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}

fn command_name(command: &LoggedCommand) -> String {
    let text = format!("{command:?}");
    text.replace(" {", "(").replace(" }", ")")
}

/// Scalar fields as `before→after`, list fields by what was added or removed. The hand
/// is left out: it changes with every draw.
fn changes(state: &Checkpoint, delta: &CheckpointDelta) -> Vec<String> {
    let mut out = Vec::new();
    let mut scalar = |name: &str, before: String, after: Option<String>| {
        if let Some(after) = after.filter(|after| *after != before) {
            out.push(format!("{name} {before}→{after}"));
        }
    };
    scalar("phase", state.phase.clone(), delta.phase.clone());
    scalar(
        "ante",
        state.ante.to_string(),
        delta.ante.map(|v| v.to_string()),
    );
    scalar(
        "money",
        state.money.to_string(),
        delta.money.map(|v| v.to_string()),
    );
    scalar(
        "score",
        state.score.to_string(),
        delta.score.map(|v| v.to_string()),
    );
    scalar(
        "hands",
        state.hands.to_string(),
        delta.hands.map(|v| v.to_string()),
    );
    scalar(
        "discards",
        state.discards.to_string(),
        delta.discards.map(|v| v.to_string()),
    );
    scalar(
        "reward",
        state.reward.to_string(),
        delta.reward.map(|v| v.to_string()),
    );
    for (name, before, after) in [
        ("jokers", &state.jokers, &delta.jokers),
        ("consumables", &state.consumables, &delta.consumables),
        ("tags", &state.tags, &delta.tags),
    ] {
        let Some(after) = after.as_ref().filter(|after| *after != before) else {
            continue;
        };
        let added: Vec<_> = after.iter().filter(|item| !before.contains(item)).collect();
        let removed: Vec<_> = before.iter().filter(|item| !after.contains(item)).collect();
        if !added.is_empty() {
            out.push(format!("+{name} {added:?}"));
        }
        if !removed.is_empty() {
            out.push(format!("-{name} {removed:?}"));
        }
        if added.is_empty() && removed.is_empty() {
            out.push(format!("{name} reordered"));
        }
    }
    out
}

fn print_explain(explain: &Explain) {
    let print = |label: &str, items: &[String]| {
        if !items.is_empty() {
            println!("        {label}: {}", items.join("; "));
        }
    };
    if let Some(blind) = &explain.blind {
        print("blind", std::slice::from_ref(blind));
    }
    if let Some(hand) = &explain.hand {
        print("hand", std::slice::from_ref(hand));
    }
    print("effects", &explain.effects);
    print("notes", &explain.notes);
    print("jokers", &explain.jokers);
    print("shop", &explain.shop);
    print("pack", &explain.pack);
}

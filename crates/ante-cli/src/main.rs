use std::process::ExitCode;

use ante_core::{
    contract::{CardKey, Command, Deck, RunConfig, RunDriver, TraceMode},
    native::{NativeReplay, NativeRun},
};

fn main() -> ExitCode {
    match execute() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn execute() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let Some(seed) = args.next() else {
        println!(
            "Usage: ante-cli SEED [--blue] start [select:ID ...] [play|discard|rank|suit|cash-out ...]"
        );
        return Ok(());
    };
    let seed = seed
        .parse()
        .map_err(|_| "Seed must be an unsigned integer.")?;
    let mut deck = Deck::Red;
    let mut commands = Vec::new();
    for token in args {
        let command = match token.as_str() {
            "--blue" if commands.is_empty() => {
                deck = Deck::Blue;
                continue;
            }
            "start" => Command::StartBlind,
            "play" => Command::Play,
            "discard" => Command::Discard,
            "rank" => Command::SortRank,
            "suit" => Command::SortSuit,
            "cash-out" => Command::CashOut,
            _ => {
                let id = token
                    .strip_prefix("select:")
                    .and_then(|id| id.parse::<usize>().ok())
                    .ok_or_else(|| format!("Unknown command: {token}"))?;
                Command::ToggleCard(CardKey(id))
            }
        };
        commands.push(command);
    }
    let config = RunConfig {
        seed,
        deck,
        ..Default::default()
    };
    let replay = NativeReplay::new(config, commands);
    let mut run = NativeRun::new(config);
    for &command in &replay.commands {
        let update = run
            .apply(command, TraceMode::Capture)
            .map_err(|error| format!("{command:?}: {error}"))?;
        if let Some(trace) = update.outcome.trace {
            for step in trace.0 {
                println!(
                    "{:?}: {} × {}",
                    step.source, step.chips_after, step.mult_after
                );
            }
        }
    }
    let view = run.view();
    println!(
        "Ante replay v{} / RNG v{} | seed {} | {:?} | {}/{} | ${}",
        replay.version,
        replay.rng_version,
        view.seed,
        view.phase,
        view.score,
        view.target,
        view.money
    );
    for card in view.cards {
        println!("#{} {} {}", card.id.0, card.rank, card.suit);
    }
    Ok(())
}

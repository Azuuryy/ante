use super::*;
use crate::contract::{RunDriver, TraceMode};
use crate::engine::EngineRun;

const CONFIG: RunConfig = RunConfig {
    seed: 7,
    deck: Deck::Red,
    goal: CampaignGoal::Standard,
    starting_money: None,
};

/// Plays a few real commands, a refused one included, and returns the log lines.
fn recorded_run() -> Vec<String> {
    let mut run = EngineRun::with_config(CONFIG);
    let (mut recorder, header) = RunRecorder::start(CONFIG);
    let mut lines = vec![header];
    let mut send = |run: &mut EngineRun, command: &dyn Fn(&RunView) -> Command| {
        let before = RunDriver::view(run);
        let command = command(&before);
        let result = RunDriver::apply(run, command, TraceMode::Capture);
        let after = RunDriver::view(run);
        lines.push(recorder.step(&before, command, result.as_ref(), &after));
    };
    send(&mut run, &|_| Command::StartBlind);
    for position in 0..5 {
        send(&mut run, &move |view| {
            Command::ToggleCard(view.cards[position].id)
        });
    }
    send(&mut run, &|_| Command::Play);
    send(&mut run, &|view| Command::ToggleCard(view.cards[0].id));
    send(&mut run, &|_| Command::Discard);
    // Refused: nothing is selected after the discard.
    send(&mut run, &|_| Command::Play);
    lines
}

#[test]
fn a_recorded_run_replays_without_divergence() {
    let lines = recorded_run();
    // Another run moves the engine's global card ids before the replay.
    let _other = EngineRun::with_config(RunConfig { seed: 99, ..CONFIG });
    let report = replay(lines.iter().map(String::as_str)).unwrap();
    assert_eq!(report.divergence, None);
    assert_eq!(report.steps, lines.len() - 1);
}

#[test]
fn a_changed_state_is_reported_at_its_step() {
    let mut lines = recorded_run();
    let mut line: LogLine = serde_json::from_str(&lines[7]).unwrap();
    let LogLine::Step(step) = &mut line else {
        panic!("a step line")
    };
    step.after.money += 1;
    let index = step.index;
    lines[7] = serde_json::to_string(&line).unwrap();
    let report = replay(lines.iter().map(String::as_str)).unwrap();
    let divergence = report.divergence.expect("a divergence");
    assert_eq!(divergence.index, index);
    assert_eq!(divergence.logged.money, divergence.replayed.money + 1);
}

#[test]
fn a_refused_command_is_logged_and_replays_refused() {
    let lines = recorded_run();
    let LogLine::Step(last) = serde_json::from_str(lines.last().unwrap()).unwrap() else {
        panic!("a step line")
    };
    assert_eq!(last.command, LoggedCommand::Play);
    assert!(last.refused.is_some());
    assert_eq!(last.after.phase, "playing");
}

#[test]
fn lines_store_positions_not_runtime_ids() {
    let lines = recorded_run();
    assert!(lines[2].contains("\"hand_index\":0"), "{}", lines[2]);
    assert!(!lines.iter().any(|line| line.contains("CardKey")));
    let LogLine::Header(header) = serde_json::from_str(&lines[0]).unwrap() else {
        panic!("a header line")
    };
    assert_eq!(header.config().unwrap(), CONFIG);
}

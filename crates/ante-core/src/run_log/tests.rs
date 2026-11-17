use super::*;
use crate::contract::{RunDriver, TraceMode};
use crate::engine::EngineRun;

const CONFIG: RunConfig = RunConfig {
    seed: 7,
    deck: Deck::Red,
    goal: CampaignGoal::Standard,
    starting_money: None,
    stake: 1,
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
        recorder.step(&before, command, result.as_ref(), &after)
    };
    lines.extend(send(&mut run, &|_| Command::StartBlind));
    for position in 0..5 {
        lines.extend(send(&mut run, &move |view| {
            Command::ToggleCard(view.cards[position].id)
        }));
    }
    lines.extend(send(&mut run, &|_| Command::Play));
    lines.extend(send(&mut run, &|view| {
        Command::ToggleCard(view.cards[0].id)
    }));
    lines.extend(send(&mut run, &|_| Command::Discard));
    // Refused: nothing is selected after the discard.
    lines.extend(send(&mut run, &|_| Command::Play));
    lines.extend(send(&mut run, &|view| {
        Command::ToggleCard(view.cards[0].id)
    }));
    lines.extend(recorder.finish());
    lines
}

fn parse(line: &str) -> LogLine {
    serde_json::from_str(line).unwrap()
}

#[test]
fn a_recorded_run_replays_without_divergence() {
    let lines = recorded_run();
    // Another run moves the engine's global card ids before the replay.
    let _other = EngineRun::with_config(RunConfig { seed: 99, ..CONFIG });
    let report = replay(lines.iter().map(String::as_str)).unwrap();
    assert_eq!(report.divergence, None);
    assert_eq!(report.steps, 11);
}

#[test]
fn selections_share_one_line_written_before_the_next_command() {
    let lines = recorded_run();
    // Header, start, 5 toggles, play, toggle, discard, play, then the trailing toggle.
    assert_eq!(lines.len(), 8);
    let LogLine::Select(group) = parse(&lines[2]) else {
        panic!("a select line: {}", lines[2])
    };
    assert_eq!(group.index, 1);
    assert_eq!(group.commands.len(), 5);
    let hand = group.after.hand.expect("the selection changed the hand");
    assert_eq!(hand.iter().filter(|card| card.ends_with('*')).count(), 5);
    assert!(
        matches!(parse(&lines[7]), LogLine::Select(_)),
        "finish writes it"
    );
}

#[test]
fn steps_store_only_what_changed_and_explain_the_hand() {
    let lines = recorded_run();
    let LogLine::Step(play) = parse(&lines[3]) else {
        panic!("a step line")
    };
    assert_eq!(play.command, LoggedCommand::Play);
    assert!(play.after.ante.is_none(), "the ante did not change");
    assert!(play.after.score.is_some());
    let explain = play.explain.expect("a played hand is explained");
    assert!(explain.hand.is_some(), "{explain:?}");
    assert!(!explain.effects.is_empty(), "{explain:?}");
    let LogLine::Step(start) = parse(&lines[1]) else {
        panic!("a step line")
    };
    assert!(start.explain.and_then(|explain| explain.blind).is_some());
    assert!(
        !lines[1].contains("\"refused\""),
        "no null fields: {}",
        lines[1]
    );
}

#[test]
fn a_changed_state_is_reported_at_its_step() {
    let mut lines = recorded_run();
    let mut line = parse(&lines[3]);
    let LogLine::Step(step) = &mut line else {
        panic!("a step line")
    };
    step.after.money = Some(step.after.money.unwrap_or(4) + 1);
    let index = step.index;
    lines[3] = serde_json::to_string(&line).unwrap();
    let report = replay(lines.iter().map(String::as_str)).unwrap();
    let divergence = report.divergence.expect("a divergence");
    assert_eq!(divergence.index, index);
    assert_eq!(divergence.logged.money, divergence.replayed.money + 1);
}

#[test]
fn a_refused_command_is_logged_and_replays_refused() {
    let lines = recorded_run();
    let LogLine::Step(last) = parse(&lines[6]) else {
        panic!("a step line")
    };
    assert_eq!(last.command, LoggedCommand::Play);
    assert!(last.refused.is_some());
}

#[test]
fn lines_store_positions_not_runtime_ids() {
    let lines = recorded_run();
    assert!(lines[2].contains("\"t0\""), "{}", lines[2]);
    assert!(!lines.iter().any(|line| line.contains("CardKey")));
    let LogLine::Header(header) = parse(&lines[0]) else {
        panic!("a header line")
    };
    assert_eq!(header.config().unwrap(), CONFIG);
    assert_eq!(header.format_version, FORMAT_VERSION);
}

/// A format 1 log: every field on every line, full card names.
#[test]
fn a_format_1_log_still_replays() {
    let mut run = EngineRun::with_config(CONFIG);
    let mut header = Header::new(CONFIG);
    header.format_version = 1;
    let mut lines = vec![serde_json::to_string(&LogLine::Header(header)).unwrap()];
    let commands: [&dyn Fn(&RunView) -> Command; 3] = [
        &|_| Command::StartBlind,
        &|view| Command::ToggleCard(view.cards[0].id),
        &|_| Command::Play,
    ];
    for (index, command) in commands.into_iter().enumerate() {
        let before = RunDriver::view(&run);
        let command = command(&before);
        let refused = RunDriver::apply(&mut run, command, TraceMode::Off)
            .err()
            .map(|error| error.to_string());
        let full = Checkpoint::of_format(&RunDriver::view(&run), 1);
        let step = Step {
            index,
            command: LoggedCommand::from_command(command, &before),
            refused,
            trace_steps: 0,
            after: CheckpointDelta::between(&Checkpoint::default(), &full),
            explain: None,
        };
        let mut value = serde_json::to_value(LogLine::Step(step)).unwrap();
        // Format 1 wrote every field, `null` refusals included.
        value["after"] = serde_json::to_value(&full).unwrap();
        value["refused"] = serde_json::Value::Null;
        lines.push(value.to_string());
    }
    assert!(lines[2].contains(" *"), "full card names: {}", lines[2]);
    let report = replay(lines.iter().map(String::as_str)).unwrap();
    assert_eq!(report.divergence, None);
}

#[test]
fn selection_codes_read_back() {
    for command in [
        LoggedCommand::ToggleCard { hand_index: 7 },
        LoggedCommand::SortRank,
        LoggedCommand::SortSuit,
        LoggedCommand::MoveCard {
            hand_index: 3,
            right: true,
        },
        LoggedCommand::MoveCard {
            hand_index: 0,
            right: false,
        },
    ] {
        let code = selection_codes::code(&command);
        assert_eq!(selection_codes::parse(&code), Some(command), "{code}");
    }
    assert_eq!(selection_codes::parse("x1"), None);
}

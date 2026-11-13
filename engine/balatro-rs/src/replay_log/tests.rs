use super::*;
use crate::stage::End;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

#[test]
fn replay_log_recomputes_seeded_game() {
    let config = Config {
        seed: Some(10037),
        ante_end: 1,
        ..Config::default()
    };
    let mut game = Game::new(config.clone());
    let seed = game.seed;
    let seed_str = game.seed_str.clone();
    game.start();

    let mut lines = vec![header_line(
        &config,
        seed,
        seed_str.as_deref(),
        ReplaySource::Player,
        None,
    )
    .unwrap()];
    let mut rng = StdRng::seed_from_u64(1);
    let mut steps = 0usize;
    while !game.is_over() && steps < 20_000 {
        let space = game.gen_action_space();
        let mask = space.to_vec();
        let legal: Vec<usize> = mask
            .iter()
            .enumerate()
            .filter_map(|(index, bit)| (*bit == 1).then_some(index))
            .collect();
        assert!(!legal.is_empty(), "no legal action at step {steps}");
        let index = legal[rng.gen_range(0..legal.len())];
        let action = space.to_action(index, &game).expect("index maps to action");
        assert_eq!(space.index_of(&action, &game), Some(index));
        game.handle_action_index(index).expect("apply action");
        lines.push(action_line(index));
        steps += 1;
    }
    assert!(
        game.is_over(),
        "seeded game did not finish after {steps} steps ({:?})",
        game.stage
    );
    lines.push(outcome_line(&game, steps).unwrap());

    let mut records = lines.iter().map(|line| parse_line(line).unwrap());
    let ReplayRecord::Header(header) = records.next().unwrap() else {
        panic!("first line must be the header");
    };
    let mut replay = game_from_header(&header);
    let mut replay_steps = 0usize;
    let mut saw_outcome = false;
    for record in records {
        match record {
            ReplayRecord::Move(step) => {
                apply_move(&mut replay, step).expect("replay move");
                replay_steps += 1;
            }
            ReplayRecord::Outcome(outcome) => {
                assert_eq!(
                    outcome.result,
                    if matches!(game.result(), Some(End::Win)) {
                        ReplayResult::Won
                    } else {
                        ReplayResult::Lost
                    }
                );
                assert_eq!(outcome.score, game.score);
                assert_eq!(outcome.steps, steps);
                saw_outcome = true;
            }
            ReplayRecord::Header(_) => panic!("unexpected extra header"),
        }
    }
    assert!(saw_outcome);
    assert_eq!(replay_steps, steps);
    assert_eq!(replay.score, game.score);
    assert_eq!(replay.is_over(), game.is_over());
    assert_eq!(replay.result(), game.result());
}

#[test]
fn deselect_is_logged_and_replays() {
    let config = Config {
        seed: Some(10037),
        ante_end: 1,
        ..Config::default()
    };
    let mut game = Game::new(config.clone());
    game.start();
    game.handle_action_index(78).expect("select blind");

    let space = game.gen_action_space();
    let select = space
        .to_vec()
        .iter()
        .position(|&bit| bit == 1)
        .expect("select a card");
    let select_action = space.to_action(select, &game).unwrap();
    let Action::SelectCard(card) = select_action.clone() else {
        panic!("expected SelectCard");
    };
    assert_eq!(
        loggable_move(&game, &select_action),
        Some(ReplayMove::Action(select))
    );
    game.handle_action(select_action).unwrap();

    let deselect = Action::DeselectCard(card);
    let logged = loggable_move(&game, &deselect).expect("deselect is loggable");
    assert_eq!(logged, ReplayMove::Deselect(select));
    game.handle_action(deselect).unwrap();

    let line = move_line(logged);
    match parse_line(&line).unwrap() {
        ReplayRecord::Move(step) => assert_eq!(step, logged),
        other => panic!("expected Move, got {other:?}"),
    }

    let mut replay = Game::new(config);
    replay.start();
    apply_move(&mut replay, ReplayMove::Action(78)).unwrap();
    apply_move(&mut replay, ReplayMove::Action(select)).unwrap();
    apply_move(&mut replay, logged).unwrap();
    assert!(replay.available.selected().is_empty());
}

#[test]
fn parse_line_rejects_unknown_version() {
    let line = r#"{"format_version":1,"seed":1,"config":{}}"#;
    let err = parse_line(line).unwrap_err();
    assert!(matches!(err, ReplayLogError::BadVersion(1)));
}

#[test]
fn abandoned_line_is_readable() {
    let mut game = Game::new(Config::default());
    game.start();
    let line = terminal_line(&game, 0, ReplayResult::Abandoned).unwrap();
    let ReplayRecord::Outcome(outcome) = parse_line(&line).unwrap() else {
        panic!("expected outcome")
    };
    assert_eq!(outcome.result, ReplayResult::Abandoned);
}

#[test]
fn headers_mark_source_explicitly() {
    let config = Config::default();
    for (agent, expected) in [
        (None, ReplaySource::Player),
        (Some("policy"), ReplaySource::Model),
        (Some("human label"), ReplaySource::Player),
    ] {
        let line = header_line(&config, 42, None, expected, agent).unwrap();
        let ReplayRecord::Header(header) = parse_line(&line).unwrap() else {
            panic!("expected header")
        };
        assert_eq!(header.source, expected);
        assert_eq!(header.config.seed, None);
    }
}

#[test]
fn text_seed_replays_from_single_header_field() {
    let config = Config {
        seed_str: Some("TEST".into()),
        ..Config::default()
    };
    let mut original = Game::new(config);
    original.start();
    let line = header_line(
        &original.config,
        original.seed,
        original.seed_str.as_deref(),
        ReplaySource::Player,
        None,
    )
    .unwrap();
    let ReplayRecord::Header(header) = parse_line(&line).unwrap() else {
        panic!("header")
    };
    assert_eq!(header.seed, ReplaySeed::Text("TEST".into()));
    assert!(header.config.seed.is_none());
    assert!(header.config.seed_str.is_none());
    let replay = game_from_header(&header);
    assert_eq!(original.seed, replay.seed);
    assert_eq!(original.seed_str, replay.seed_str);
}

#[test]
fn writer_flushes_a_live_run_and_finishes_once() {
    let mut game = Game::new(Config {
        seed: Some(42),
        ..Config::default()
    });
    game.start();
    let mut writer =
        ReplayWriter::new(Vec::new(), &game, ReplaySource::Model, Some("gym")).unwrap();
    writer.record_move(ReplayMove::Action(78)).unwrap();
    game.handle_action_index(78).unwrap();
    writer
        .record_outcome(&game, ReplayResult::Abandoned)
        .unwrap();
    writer
        .record_outcome(&game, ReplayResult::Abandoned)
        .unwrap();
    assert!(writer.record_move(ReplayMove::Action(78)).is_err());
    let text = String::from_utf8(writer.writer).unwrap();
    let lines: Vec<_> = text.lines().collect();
    assert_eq!(lines.len(), 3);
    let ReplayRecord::Header(header) = parse_line(lines[0]).unwrap() else {
        panic!("header")
    };
    assert_eq!(header.source, ReplaySource::Model);
    assert_eq!(header.config.seed, None);
    let ReplayRecord::Outcome(outcome) = parse_line(lines[2]).unwrap() else {
        panic!("outcome")
    };
    assert_eq!(outcome.result, ReplayResult::Abandoned);
    assert_eq!(outcome.steps, 1);
}

use super::*;
use ante_core::contract::{CardTrigger, ContentId, HandKind, JokerKey};

#[test]
fn a_step_shows_its_popups_in_the_original_order() {
    let mut step = card(0, (10, 21));
    step.mult_before = 4.0;
    step.mult_after = 12.0;
    step.xmult = Some(150);
    step.money = 3;
    assert_eq!(
        popups(&step, None),
        vec![
            (Popup::Chips(11), CHIPS_POPUP),
            (Popup::Mult(4), MULT_POPUP),
            (Popup::XMult(150), MULT_POPUP),
            (Popup::Money(3), MONEY_POPUP),
        ]
    );
}

#[test]
fn a_joker_xmult_is_one_joker_popup() {
    let step = ScoreStep {
        source: ScoreSource::Joker {
            id: JokerKey(1),
            kind: ContentId("j_duo"),
            name: "The Duo",
        },
        chips_before: 20,
        chips_after: 20,
        mult_before: 3.0,
        mult_after: 6.0,
        retrigger: false,
        xmult: Some(200),
        money: 0,
    };
    assert_eq!(popups(&step, None), vec![(Popup::XMult(200), JOKER_POPUP)]);
}

#[test]
fn the_current_popup_follows_the_clock() {
    let mut step = card(0, (10, 21));
    step.mult_after = 5.0;
    let trace = ScoreTrace(vec![step]);
    let mut playback = Playback::new(trace, 1);
    let start = playback.starts[0];
    playback.clock = start + 0.1;
    assert_eq!(playback.current_popup(), Some((0, 0, Popup::Chips(11))));
    playback.clock = start + CHIPS_POPUP + 0.1;
    assert_eq!(playback.current_popup(), Some((0, 1, Popup::Mult(4))));
}

#[test]
fn saved_speed_factors_round_trip() {
    for speed in Speed::ALL {
        assert_eq!(Speed::from_factor(speed.factor()), Some(speed));
    }
    assert_eq!(Speed::from_factor(3.0), None);
}

fn card(index: usize, chips: (usize, usize)) -> ScoreStep {
    ScoreStep {
        source: ScoreSource::Card {
            trigger: CardTrigger::Played,
            id: CardKey(index),
            rank: 'A',
            suit: "Spades",
        },
        chips_before: chips.0,
        chips_after: chips.1,
        mult_before: 1.0,
        mult_after: 1.0,
        retrigger: false,
        xmult: None,
        money: 0,
    }
}

fn five_card_chips_trace() -> ScoreTrace {
    let mut steps = vec![ScoreStep {
        source: ScoreSource::HandLevel(HandKind::Flush),
        chips_before: 0,
        chips_after: 35,
        mult_before: 0.0,
        mult_after: 4.0,
        retrigger: false,
        xmult: None,
        money: 0,
    }];
    steps.extend((0..5).map(|index| card(index, (35 + index * 11, 46 + index * 11))));
    ScoreTrace(steps)
}

fn run_until_done(playback: &mut Playback, speed: Speed) -> f32 {
    let frame = Duration::from_secs_f32(1.0 / 60.0);
    let mut real = 0.0;
    while !playback.advance(frame, speed) {
        real += frame.as_secs_f32();
        assert!(real < 120.0, "playback never finished");
    }
    real
}

#[test]
fn five_chip_cards_last_as_long_as_the_original_sequence() {
    let playback = Playback::new(five_card_chips_trace(), 5);
    // 0.4 + 5 x 0.1 + 0.2 + 5 x 0.1 + 0.4, five chip popups, then
    // 5 x 0.1 + 0.4 + 0.8 + 0.5 + 0.3 + 5 x 0.1.
    let expected = 2.0 + 5.0 * CHIPS_POPUP + 3.0;
    assert!((playback.duration() - expected).abs() < 1e-4);
}

#[test]
fn real_time_divides_by_the_speed_factor() {
    let one = run_until_done(&mut Playback::new(five_card_chips_trace(), 5), Speed(1));
    let four = run_until_done(&mut Playback::new(five_card_chips_trace(), 5), Speed(3));
    assert!((one - 8.75).abs() < 0.1, "1x took {one}");
    assert!((four - 8.75 / 4.0).abs() < 0.1, "4x took {four}");
}

#[test]
fn long_sequences_accelerate_like_the_original() {
    let joker = |index| ScoreStep {
        source: ScoreSource::Joker {
            id: JokerKey(index),
            kind: ContentId("j_joker"),
            name: "Joker",
        },
        chips_before: 10,
        chips_after: 10,
        mult_before: index as f64,
        mult_after: index as f64 + 4.0,
        retrigger: false,
        xmult: None,
        money: 0,
    };
    let trace = ScoreTrace((0..40).map(joker).collect());
    let mut playback = Playback::new(trace, 5);
    let game = playback.duration();
    let real = run_until_done(&mut playback, Speed::default());
    assert!(real < game * 0.75, "{real}s for {game}s of game time");
}

#[test]
fn speeds_double_from_half_to_128_and_wrap() {
    let labels: Vec<_> = Speed::ALL.iter().map(|speed| speed.label()).collect();
    assert_eq!(
        labels,
        ["x0.5", "x1", "x2", "x4", "x8", "x16", "x32", "x64", "x128"]
    );
    assert_eq!(Speed::default().factor(), 1.0);
    assert_eq!(Speed::ALL[8].next(), Speed::ALL[0]);
}

#[test]
fn stages_and_focus_follow_the_clock() {
    let mut playback = Playback::new(five_card_chips_trace(), 5);
    assert_eq!(playback.stage(), Stage::Intro);
    assert_eq!(playback.focused_card(), None);
    playback.advance(Duration::from_secs_f32(2.05), Speed::default());
    assert_eq!(playback.stage(), Stage::Scoring);
    assert_eq!(playback.focused_card(), Some(CardKey(0)));
    assert_eq!(playback.score_values(), Some((46, 1.0)));
    playback.advance(Duration::from_secs_f32(4.0), Speed::default());
    assert_eq!(playback.stage(), Stage::Total);
    assert_eq!(playback.focused_card(), None);
}

#[test]
fn a_retrigger_shows_its_own_popup_once() {
    let first = card(0, (0, 10));
    let mut again = card(0, (10, 20));
    again.retrigger = true;
    assert_eq!(
        step_duration(&again, Some(&first)),
        CHIPS_POPUP + JOKER_POPUP
    );
    let mut twice = again.clone();
    twice.chips_before = 20;
    twice.chips_after = 30;
    assert_eq!(step_duration(&twice, Some(&again)), CHIPS_POPUP);
}

#[test]
fn a_joker_effect_on_a_card_pops_on_the_card() {
    let mut step = card(0, (10, 40));
    step.source = ScoreSource::JokerOnCard {
        joker: JokerKey(3),
        kind: ContentId("j_scary_face"),
        name: "Scary Face",
        card: CardKey(0),
    };
    step.retrigger = true;
    // Card delays, and no second "Again!" after the card's own repetition step.
    assert_eq!(
        popups(&step, Some(&card(0, (0, 10)))),
        vec![(Popup::Chips(30), CHIPS_POPUP)]
    );
}

#[test]
fn a_refused_hand_shows_not_allowed_over_the_play_area() {
    let step = ScoreStep {
        source: ScoreSource::NotAllowed,
        chips_before: 0,
        chips_after: 0,
        mult_before: 0.0,
        mult_after: 0.0,
        retrigger: false,
        xmult: None,
        money: 0,
    };
    assert_eq!(
        popups(&step, None),
        vec![(Popup::NotAllowed, NOT_ALLOWED_POPUP)]
    );
}

#[test]
fn a_joker_upgrade_is_one_joker_message() {
    let step = ScoreStep {
        source: ScoreSource::JokerMessage {
            id: JokerKey(3),
            kind: ContentId("j_wee"),
            name: "Wee Joker",
            message: "Upgrade!",
            ink: ante_core::contract::MessageInk::Chips,
        },
        chips_before: 40,
        chips_after: 40,
        mult_before: 2.0,
        mult_after: 2.0,
        retrigger: false,
        xmult: None,
        money: 0,
    };
    assert_eq!(
        popups(&step, None),
        vec![(
            Popup::Message("Upgrade!", ante_core::contract::MessageInk::Chips),
            JOKER_POPUP
        )]
    );
}

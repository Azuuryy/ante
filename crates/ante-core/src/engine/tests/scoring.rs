use super::*;
use balatro_rs::joker::Jokers;

/// Every joker the engine can offer, alone in a fresh run.
fn offered_jokers() -> Vec<Jokers> {
    use balatro_rs::joker::JokerEffects;
    balatro_rs::joker::joker_catalog()
        .filter(|joker| joker.is_implemented())
        .collect()
}

/// Implemented jokers whose original text has a "(Currently ...)" line.
const ORIGINAL_CURRENTLY: &[&str] = &[
    "j_abstract",
    "j_blue_joker",
    "j_bull",
    "j_castle",
    "j_drivers_license",
    "j_erosion",
    "j_green_joker",
    "j_hit_the_road",
    "j_obelisk",
    "j_ride_the_bus",
    "j_steel_joker",
    "j_stencil",
    "j_stone",
    "j_trousers",
    "j_wee",
    "j_yorick",
];

#[test]
fn every_offered_joker_has_its_own_description() {
    for joker in offered_jokers() {
        let text = joker_description(joker.id());
        assert!(!text.is_empty(), "{} has no description", joker.id());
        assert_ne!(
            text,
            joker.desc(),
            "{} falls back to the engine text",
            joker.id()
        );
    }
}

#[test]
fn every_scaling_joker_shows_its_current_value() {
    let game = start().game;
    let offered: Vec<_> = offered_jokers().iter().map(|joker| joker.id()).collect();
    for id in ORIGINAL_CURRENTLY {
        assert!(offered.contains(id), "{id} is no longer offered");
        let joker = Jokers::from_id(id).unwrap();
        assert!(
            joker_current(&joker, &game).is_some(),
            "{id} has no Currently line"
        );
    }
    for id in ["j_banner", "j_mystic_summit"] {
        let joker = Jokers::from_id(id).unwrap();
        assert!(joker_current(&joker, &game).is_some(), "{id}");
    }
}

#[test]
fn every_offered_joker_scores_a_hand() {
    for joker in offered_jokers() {
        let id = joker.id();
        let mut run = start();
        run.game.jokers = vec![joker];
        let before = run.game.clone();
        run.game
            .effect_registry
            .register_jokers(before.jokers.clone(), &before);
        for index in 0..5 {
            let card = run.view().cards[index].id;
            run.apply(Command::ToggleCard(card)).unwrap();
        }
        let outcome = run.apply(Command::Play);
        assert!(outcome.is_ok(), "{id}: {outcome:?}");
        assert!(
            run.view().score > 0 || run.view().phase != Phase::Playing,
            "{id} scored nothing"
        );
        assert!(run.view().held_jokers.len() <= 1, "{id} multiplied itself");
    }
}

#[test]
fn banner_and_mystic_summit_follow_the_discards_left() {
    let mut run = start();
    let banner = Jokers::from_id("j_banner").unwrap();
    let summit = Jokers::from_id("j_mystic_summit").unwrap();
    let left = run.game.discards();
    assert_eq!(
        joker_current(&banner, &run.game),
        Some(CurrentValue::Chips(30 * left))
    );
    assert_eq!(
        joker_current(&summit, &run.game),
        Some(CurrentValue::Mult(0))
    );
    run.game.discards_remaining = 0;
    assert_eq!(
        joker_current(&summit, &run.game),
        Some(CurrentValue::Mult(15))
    );
}

#[test]
fn consumables_name_the_most_cards_they_take() {
    assert_eq!(
        with_selection("Effect.", true, 2),
        "Effect. Select at most 2 cards."
    );
    assert_eq!(
        with_selection("Effect.", true, 1),
        "Effect. Select at most 1 card."
    );
    assert_eq!(with_selection("Effect.", true, 0), "Effect.");
    assert_eq!(with_selection("Effect.", false, 3), "Effect.");
}

#[test]
fn pack_cards_keep_their_enhancement_edition_and_seal() {
    use balatro_rs::card::{Card, Edition, Enhancement, Seal, Suit, Value};
    let mut card = Card::new(Value::Ace, Suit::Heart);
    card.enhancement = Some(Enhancement::Gold);
    card.edition = Edition::Foil;
    card.seal = Some(Seal::Red);
    let ShopArt::Card { look, .. } = pack_content_art(&PackContent::PlayingCard(card)) else {
        panic!("a playing card projects to card art");
    };
    assert_eq!(look.enhancement, Some("Gold"));
    assert_eq!(look.edition, "Foil");
    assert_eq!(look.seal, Some("Red"));
}

#[test]
fn an_open_pack_family_follows_its_contents() {
    use balatro_rs::card::{Card, Suit, Value};
    let cards = [PackContent::PlayingCard(Card::new(Value::Two, Suit::Club))];
    assert_eq!(open_pack_family(&cards), PackFamily::Standard);
    assert_eq!(open_pack_family(&[]), PackFamily::Spectral);
}

#[test]
fn trace_steps_keep_the_xmult_factor_and_money() {
    let native = NativeTrace(vec![balatro_rs::score::ScoreStep {
        source: NativeSource::HandLevel(balatro_rs::rank::HandRank::OnePair),
        chips_before: 10,
        chips_after: 10,
        mult_before: 4.0,
        mult_after: 6.0,
        retrigger: false,
        xmult: Some(1.5),
        money: 3,
    }]);
    let step = &map_trace(native).0[0];
    assert_eq!(step.xmult, Some(150));
    assert_eq!(step.money, 3);
}

fn start() -> EngineRun {
    let mut run = EngineRun::new(0);
    run.apply(Command::StartBlind).unwrap();
    run
}

fn select_first(run: &mut EngineRun) {
    let id = run.view().cards[0].id;
    run.apply(Command::ToggleCard(id)).unwrap();
}

#[test]
fn traced_play_matches_plain_state_and_rng() {
    let mut plain = start();
    let mut first = plain.game.available.cards()[0];
    first.enhancement = Some(balatro_rs::card::Enhancement::Lucky);
    let mut snapshot: serde_json::Value = serde_json::from_str(&plain.snapshot().unwrap()).unwrap();
    snapshot["available"]["cards"][0][0] = serde_json::to_value(first).unwrap();
    plain.game = Game::from_json(&snapshot.to_string()).unwrap();
    assert_eq!(plain.game.available.cards()[0], first);
    plain
        .game
        .jokers
        .push(Jokers::HangingChad(Default::default()));
    #[cfg(feature = "extra-jokers")]
    plain.game.jokers.push(Jokers::Misprint(Default::default()));
    let before = plain.game.clone();
    plain
        .game
        .effect_registry
        .register_jokers(before.jokers.clone(), &before);
    select_first(&mut plain);
    let mut traced = EngineRun {
        game: plain.game.clone(),
        last_hand_score: None,
        revision: plain.revision,
        skipped_blinds: plain.skipped_blinds.clone(),
        stats: plain.stats,
        sort_by_suit: plain.sort_by_suit,
        debug: plain.debug,
    };
    plain.apply(Command::Play).unwrap();
    let outcome = traced.apply_traced(Command::Play, true).unwrap();
    let trace = outcome.trace.unwrap();
    assert!(trace.0.iter().any(|step| step.retrigger));
    assert_eq!(plain.view(), traced.view());
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&plain.snapshot().unwrap()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&traced.snapshot().unwrap()).unwrap(),
    );
    for _ in 0..64 {
        assert_eq!(plain.game.prob_roll(1, 2), traced.game.prob_roll(1, 2));
    }
}

#[cfg(feature = "extra-jokers")]
#[test]
fn golden_joker_rewards_the_cleared_blind_once() {
    let mut run = start();
    run.game
        .jokers
        .push(Jokers::GoldenJoker(Default::default()));
    let game = run.game.clone();
    run.game
        .effect_registry
        .register_jokers(game.jokers.clone(), &game);
    select_first(&mut run);
    let money = run.view().money;
    run.game.score = run.game.required_score() - 1;
    run.apply_traced(Command::Play, false).unwrap();
    let view = run.view();
    assert_eq!(view.phase, Phase::Reward);
    // Paid at cash out on its own row, like `calculate_dollar_bonus` in evaluate_round.
    assert_eq!(view.money, money);
    let golden = view.reward_rows.iter().find(|row| {
        matches!(
            row.kind,
            RewardKind::Joker {
                name: "Golden Joker",
                ..
            }
        )
    });
    assert_eq!(golden.map(|row| row.dollars), Some(4));
    assert_eq!(
        view.reward,
        view.reward_rows
            .iter()
            .map(|row| row.dollars)
            .sum::<usize>()
    );
    run.apply(Command::CashOut).unwrap();
    assert_eq!(run.view().money, money + view.reward);
    assert!(run.view().reward_rows.is_empty());
}

#[cfg(feature = "extra-jokers")]
#[test]
fn ice_cream_decreases_after_the_score_then_expires() {
    let mut run = start();
    let mut joker = Jokers::IceCream(Default::default());
    joker.state_mut().counter = 18.0;
    run.game.jokers.push(joker);
    for expected in [10, 5] {
        select_first(&mut run);
        let trace = run
            .apply_traced(Command::Play, false)
            .unwrap()
            .trace
            .unwrap();
        assert!(trace.0.iter().any(|step| {
            matches!(
                step.source,
                ScoreSource::Joker {
                    kind: ContentId("j_ice_cream"),
                    ..
                }
            ) && step.chips_after - step.chips_before == expected
        }));
    }
    assert!(run.game.jokers.is_empty());
}

#[cfg(feature = "extra-jokers")]
#[test]
fn misprint_rolls_vary_and_stay_in_range() {
    use balatro_rs::{
        card::{Card, Suit, Value},
        hand::SelectHand,
    };
    let mut game = start().game;
    game.jokers.push(Jokers::Misprint(Default::default()));
    let hand = SelectHand::new(vec![Card::new(Value::Ace, Suit::Spade)])
        .best_hand()
        .unwrap();
    let results: std::collections::BTreeSet<_> = (0..64)
        .map(|_| {
            game.calc_score_traced(hand.clone())
                .1
                .0
                .iter()
                .find(|step| matches!(step.source, NativeSource::Joker(Jokers::Misprint(_))))
                .map_or(0, |step| (step.mult_after - step.mult_before) as usize)
        })
        .collect();
    assert!(results.len() > 1);
    assert!(results.iter().all(|&mult| mult <= 23));
}

#[test]
fn per_card_joker_effects_reach_the_trace_on_their_card() {
    let mut run = start();
    run.game.jokers = vec![
        Jokers::ScaryFace(Default::default()),
        Jokers::Pareidolia(Default::default()),
    ];
    let before = run.game.clone();
    run.game
        .effect_registry
        .register_jokers(before.jokers.clone(), &before);
    select_first(&mut run);
    let played = run
        .view()
        .cards
        .iter()
        .find(|card| card.selected)
        .unwrap()
        .id;
    let outcome = run.apply_traced(Command::Play, false).unwrap();
    let trace = outcome.trace.unwrap();
    assert!(trace.0.iter().any(|step| matches!(
        step.source,
        ScoreSource::JokerOnCard { name: "Scary Face", card, .. } if card == played
    )));
}

#[test]
fn a_cleared_blind_lists_its_reward_rows_in_the_original_order() {
    let mut run = start();
    run.game.money = 12;
    select_first(&mut run);
    run.game.score = run.game.required_score() - 1;
    run.apply(Command::Play).unwrap();
    let view = run.view();
    assert_eq!(view.phase, Phase::Reward);
    // `evaluate_round`: the Blind, $1 per hand left, then $1 per $5 held.
    let rows: Vec<_> = view
        .reward_rows
        .iter()
        .map(|row| (row.kind.clone(), row.dollars))
        .collect();
    assert_eq!(
        rows,
        vec![
            (RewardKind::Blind, 3),
            (RewardKind::Hands { count: 3 }, 3),
            (RewardKind::Interest, 2),
        ]
    );
    assert_eq!(view.reward, 8);
}

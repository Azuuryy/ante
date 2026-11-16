//! Lot 3, batch 4a: Jokers that change the rules of the round.

use super::*;
use crate::event::{GameEvent, JokerNote};

fn game_with(jokers: Vec<Jokers>) -> Game {
    let mut g = Game::default();
    g.start();
    g.money += 1000;
    for mut joker in jokers {
        joker.set_instance_id(crate::joker::mint_joker_id());
        g.stage = Stage::Shop();
        g.shop.jokers.push(joker.clone());
        g.buy_joker(joker).unwrap();
    }
    g.stage = Stage::Blind(Blind::Small);
    g.blind = Some(Blind::Small);
    g.take_events();
    g
}

fn notes(g: &mut Game) -> Vec<JokerNote> {
    g.take_events()
        .into_iter()
        .filter_map(|event| match event {
            GameEvent::JokerMessage { note, .. } => Some(note),
            _ => None,
        })
        .collect()
}

#[test]
fn hand_size_and_round_limits_follow_the_jokers() {
    let bare = game_with(vec![]);
    let (size, plays, discards) = (
        bare.hand_size(),
        bare.plays_per_round(),
        bare.discards_per_round(),
    );
    let g = game_with(vec![
        Jokers::Juggler(Default::default()),
        Jokers::Troubadour(Default::default()),
        Jokers::MerryAndy(Default::default()),
        Jokers::Drunkard(Default::default()),
    ]);
    assert_eq!(g.hand_size(), size + 1 + 2 - 1);
    assert_eq!(g.plays_per_round(), plays - 1);
    assert_eq!(g.discards_per_round(), discards + 3 + 1);
}

#[test]
fn oops_all_sixes_doubles_probabilities() {
    // 1 in 2 doubled is certain: Business Card always pays on a face card.
    let mut g = game_with(vec![
        Jokers::OopsAllSixes(Default::default()),
        Jokers::BusinessCard(Default::default()),
    ]);
    g.money = 0;
    for _ in 0..20 {
        g.calc_score(
            SelectHand::new(vec![Card::new(Value::King, Suit::Spade)])
                .best_hand()
                .unwrap(),
        );
    }
    assert_eq!(g.money, 40);
}

#[test]
fn chaos_the_clown_gives_one_free_reroll_per_shop() {
    let mut g = game_with(vec![Jokers::ChaosTheClown(Default::default())]);
    g.stage = Stage::Shop();
    g.free_rerolls = 1;
    let (money, cost) = (g.money, g.reroll_cost);
    assert_eq!(g.current_reroll_cost(), 0);
    g.reroll().unwrap();
    assert_eq!((g.money, g.reroll_cost), (money, cost), "the free one");
    g.reroll().unwrap();
    assert_eq!(g.money, money - cost);
}

#[test]
fn burglar_trades_discards_for_hands() {
    let mut g = game_with(vec![Jokers::Burglar(Default::default())]);
    g.plays = 4;
    g.discards_remaining = 3;
    g.jokers_set_blind(Blind::Small);
    assert_eq!((g.plays, g.discards_remaining), (7, 0));
    assert_eq!(notes(&mut g), [JokerNote::HandsPlus(3)]);
}

#[test]
fn mr_bones_saves_a_run_scored_to_a_quarter() {
    let mut g = game_with(vec![Jokers::MrBones(Default::default())]);
    g.plays = 0;
    let required = g.required_score();
    g.score = 0;
    // A quarter of the blind, scored by the last hand.
    assert!(g.handle_score(required / 4).is_ok());
    assert!(!matches!(g.stage, Stage::End(_)), "the run goes on");
    assert!(g.jokers.is_empty());
    assert_eq!(notes(&mut g), [JokerNote::Saved]);
}

#[test]
fn splash_makes_every_played_card_score() {
    let pair_and_kicker = || {
        vec![
            Card::new(Value::King, Suit::Spade),
            Card::new(Value::King, Suit::Heart),
            Card::new(Value::Two, Suit::Club),
        ]
    };
    let mut bare = game_with(vec![]);
    let plain = bare.calc_score(SelectHand::new(pair_and_kicker()).best_hand().unwrap());
    let mut g = game_with(vec![Jokers::Splash(Default::default())]);
    let splashed = g.calc_score(SelectHand::new(pair_and_kicker()).best_hand().unwrap());
    // The Two's 2 chips now score too: Pair × 2 Mult.
    assert_eq!(splashed, plain + 2 * 2);
}

fn rules(four_fingers: bool, shortcut: bool, smeared: bool) -> crate::hand::HandRules {
    crate::hand::HandRules {
        four_fingers,
        shortcut,
        smeared,
    }
}

fn made(cards: &[(Value, Suit)], rules: crate::hand::HandRules) -> HandRank {
    SelectHand::new(cards.iter().map(|(v, s)| Card::new(*v, *s)).collect())
        .best_hand_with(rules)
        .unwrap()
        .rank
}

#[test]
fn four_fingers_makes_flushes_and_straights_of_four() {
    let flush4 = [
        (Value::Two, Suit::Heart),
        (Value::Five, Suit::Heart),
        (Value::Nine, Suit::Heart),
        (Value::King, Suit::Heart),
        (Value::Ace, Suit::Club),
    ];
    assert_eq!(
        made(&flush4, rules(false, false, false)),
        HandRank::HighCard
    );
    assert_eq!(made(&flush4, rules(true, false, false)), HandRank::Flush);
    let straight4 = [
        (Value::Five, Suit::Heart),
        (Value::Six, Suit::Club),
        (Value::Seven, Suit::Spade),
        (Value::Eight, Suit::Diamond),
    ];
    assert_eq!(
        made(&straight4, rules(true, false, false)),
        HandRank::Straight
    );
}

#[test]
fn shortcut_lets_a_straight_skip_one_rank() {
    let gapped = [
        (Value::Two, Suit::Heart),
        (Value::Four, Suit::Club),
        (Value::Five, Suit::Spade),
        (Value::Six, Suit::Diamond),
        (Value::Eight, Suit::Heart),
    ];
    assert_eq!(
        made(&gapped, rules(false, false, false)),
        HandRank::HighCard
    );
    assert_eq!(made(&gapped, rules(false, true, false)), HandRank::Straight);
}

#[test]
fn smeared_joker_joins_the_suits_of_a_colour() {
    let red = [
        (Value::Two, Suit::Heart),
        (Value::Five, Suit::Diamond),
        (Value::Nine, Suit::Heart),
        (Value::Jack, Suit::Diamond),
        (Value::King, Suit::Heart),
    ];
    assert_eq!(made(&red, rules(false, false, false)), HandRank::HighCard);
    assert_eq!(made(&red, rules(false, false, true)), HandRank::Flush);
    let mut g = game_with(vec![Jokers::SmearedJoker(Default::default())]);
    assert!(g.is_suit(&Card::new(Value::Two, Suit::Diamond), Suit::Heart));
    let mut stone = Card::new(Value::Two, Suit::Heart);
    stone.enhancement = Some(Enhancement::Stone);
    assert!(!g.is_suit(&stone, Suit::Heart), "a Stone card has no suit");
    g.take_events();
}

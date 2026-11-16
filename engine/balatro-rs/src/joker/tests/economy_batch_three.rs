//! Lot 3, batch 3: economy Jokers.

use super::*;
use crate::event::{GameEvent, JokerNote};
use crate::planet::Planets;
use balatro_types::joker::SelectorValue;

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
    g.money = 0;
    g.take_events();
    g
}

fn bonus(g: &mut Game) -> Vec<usize> {
    g.calc_reward(Blind::Small)
        .unwrap()
        .jokers
        .into_iter()
        .map(|(_, dollars)| dollars)
        .collect()
}

#[test]
fn cloud_9_pays_per_nine_in_the_deck() {
    let mut g = game_with(vec![Jokers::Cloud9(Default::default())]);
    // A standard deck holds four 9s.
    assert_eq!(bonus(&mut g), [4]);
}

#[test]
fn rocket_grows_after_each_boss() {
    let mut g = game_with(vec![Jokers::Rocket(Default::default())]);
    assert_eq!(bonus(&mut g), [1]);
    g.blind = Some(Blind::Boss);
    g.handle_round_end();
    assert_eq!(bonus(&mut g), [3]);
    let upgraded = g.take_events().into_iter().any(|event| {
        matches!(
            event,
            GameEvent::JokerMessage {
                note: JokerNote::UpgradeMoney,
                ..
            }
        )
    });
    assert!(upgraded);
}

#[test]
fn satellite_counts_distinct_planets() {
    let mut g = game_with(vec![Jokers::Satellite(Default::default())]);
    assert!(bonus(&mut g).is_empty(), "nothing used yet, no row");
    g.planets_used.insert(Planets::Pluto);
    g.planets_used.insert(Planets::Mars);
    assert_eq!(bonus(&mut g), [2]);
}

#[test]
fn delayed_gratification_pays_only_without_discards() {
    let mut g = game_with(vec![Jokers::DelayedGratification(Default::default())]);
    let left = g.discards_per_round();
    g.discards_remaining = left;
    assert_eq!(bonus(&mut g), [2 * left]);
    g.discards_remaining = left - 1;
    assert!(bonus(&mut g).is_empty());
}

#[test]
fn to_the_moon_adds_a_dollar_per_five_of_interest() {
    let mut g = game_with(vec![Jokers::ToTheMoon(Default::default())]);
    g.money = 12;
    // Two $5 chunks, $2 each.
    assert_eq!(g.calc_reward(Blind::Small).unwrap().interest, 4);
}

#[test]
fn egg_gains_sell_value_each_round() {
    let mut g = game_with(vec![Jokers::Egg(Default::default())]);
    let before = g.joker_sell_value(&g.jokers[0]);
    g.handle_round_end();
    assert_eq!(g.joker_sell_value(&g.jokers[0]), before + 3);
}

#[test]
fn to_do_list_pays_for_its_hand_and_picks_another() {
    let mut g = game_with(vec![Jokers::ToDoList(Default::default())]);
    g.jokers[0].state_mut().selector = Some(SelectorValue::Hand(HandRank::HighCard));
    g.calc_score(
        SelectHand::new(vec![Card::new(Value::Ace, Suit::Spade)])
            .best_hand()
            .unwrap(),
    );
    assert_eq!(g.money, 4);
    g.roll_card_targets(true);
    let Some(SelectorValue::Hand(next)) = g.jokers[0].state().selector else {
        panic!("a hand is listed")
    };
    assert_ne!(next, HandRank::HighCard);
    assert!(
        !matches!(next, HandRank::FlushFive | HandRank::RoyalFlush),
        "secret hands wait"
    );
}

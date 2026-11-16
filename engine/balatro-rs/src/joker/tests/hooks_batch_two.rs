//! Lot 3, batch 2b: Jokers on the blind, sales, added cards and the hand's edges.

use super::*;
use crate::consumable::Consumable;
use crate::event::{GameEvent, JokerNote};
use crate::tarot::Tarot;

fn game_with(jokers: Vec<Jokers>, stage: Stage) -> Game {
    let mut g = Game::default();
    g.start();
    g.money += 1000;
    for mut joker in jokers {
        joker.set_instance_id(crate::joker::mint_joker_id());
        g.stage = Stage::Shop();
        g.shop.jokers.push(joker.clone());
        g.buy_joker(joker).unwrap();
    }
    g.stage = stage;
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

fn score(g: &mut Game, cards: Vec<Card>) -> usize {
    g.calc_score(SelectHand::new(cards).best_hand().unwrap())
}

#[test]
fn hiker_gives_scored_cards_permanent_chips() {
    let mut g = game_with(
        vec![Jokers::Hiker(Default::default())],
        Stage::Blind(Blind::Small),
    );
    g.deal();
    let ace = g.available.cards()[0];
    score(&mut g, vec![ace]);
    let after = g.available.cards()[0];
    assert_eq!(after.perma_bonus, 5);
    // The bonus scores with the card from then on.
    let mut bare = game_with(vec![], Stage::Blind(Blind::Small));
    let mut plain = after;
    plain.perma_bonus = 0;
    assert_eq!(
        score(&mut bare, vec![after]),
        score(&mut bare, vec![plain]) + 5
    );
}

#[test]
fn vampire_drains_enhancements_before_they_score() {
    let mut g = game_with(
        vec![Jokers::Vampire(Default::default())],
        Stage::Blind(Blind::Small),
    );
    let mut ace = Card::new(Value::Ace, Suit::Spade);
    ace.enhancement = Some(Enhancement::Bonus);
    // The Bonus is gone before scoring: High Card Ace 16 chips × X1.1.
    assert_eq!(score(&mut g, vec![ace]), (16.0_f64 * 1.1).floor() as usize);
    assert!((g.jokers[0].state().counter - 0.1).abs() < 1e-6);
}

#[test]
fn seltzer_retriggers_then_is_drunk_after_ten_hands() {
    let mut g = game_with(
        vec![Jokers::Seltzer(Default::default())],
        Stage::Blind(Blind::Small),
    );
    // High Card Ace scored twice: (5 + 11 + 11) × 1.
    assert_eq!(score(&mut g, vec![Card::new(Value::Ace, Suit::Spade)]), 27);
    for left in (1..10).rev() {
        g.wear_seltzer();
        assert_eq!(notes(&mut g), [JokerNote::Remaining(left)]);
    }
    g.wear_seltzer();
    assert_eq!(notes(&mut g), [JokerNote::Drank]);
    assert!(g.jokers.is_empty());
}

#[test]
fn hologram_grows_with_cards_added_to_the_deck() {
    let mut g = game_with(vec![Jokers::Hologram(Default::default())], Stage::Shop());
    g.playing_cards_added(2);
    assert_eq!(notes(&mut g), [JokerNote::XMult(150)]);
}

#[test]
fn campfire_grows_on_sales_and_resets_after_a_boss() {
    let mut g = game_with(vec![Jokers::Campfire(Default::default())], Stage::Shop());
    g.consumables.push(Consumable::Tarot(Tarot::Hermit));
    g.sell_consumable(0).unwrap();
    assert_eq!(notes(&mut g), [JokerNote::UpgradePlain]);
    g.blind = Some(Blind::Boss);
    g.handle_round_end();
    assert_eq!(notes(&mut g), [JokerNote::Reset]);
    assert_eq!(g.jokers[0].state().counter, 0.0);
}

#[test]
fn ceremonial_dagger_slices_the_joker_to_its_right() {
    let mut g = game_with(
        vec![
            Jokers::CeremonialDagger(Default::default()),
            Jokers::TheJoker(Default::default()),
        ],
        Stage::PreBlind(),
    );
    g.jokers_set_blind(Blind::Small);
    assert_eq!(g.jokers.len(), 1, "The Joker was sliced");
    // The Joker sells for $1: +2 Mult.
    assert_eq!(notes(&mut g), [JokerNote::MultPlus(2)]);
}

#[test]
fn madness_grows_and_destroys_on_small_and_big_blinds_only() {
    let mut g = game_with(
        vec![
            Jokers::Madness(Default::default()),
            Jokers::TheJoker(Default::default()),
        ],
        Stage::PreBlind(),
    );
    g.jokers_set_blind(Blind::Boss);
    assert_eq!(g.jokers.len(), 2);
    assert!(notes(&mut g).is_empty());
    g.jokers_set_blind(Blind::Big);
    assert_eq!(g.jokers.len(), 1);
    assert_eq!(notes(&mut g), [JokerNote::XMult(150)]);
}

//! Lot 3, batch 5a: Jokers that create cards or level up hands.

use super::*;
use crate::consumable::Consumable;
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

fn score(g: &mut Game, cards: Vec<Card>) -> usize {
    g.calc_score(SelectHand::new(cards).best_hand().unwrap())
}

fn straight(suits: [Suit; 5]) -> Vec<Card> {
    [
        Value::Ace,
        Value::Two,
        Value::Three,
        Value::Four,
        Value::Five,
    ]
    .into_iter()
    .zip(suits)
    .map(|(value, suit)| Card::new(value, suit))
    .collect()
}

#[test]
fn eight_ball_makes_tarots_from_scored_eights() {
    let mut g = game_with(vec![Jokers::EightBall(Default::default())]);
    g.prob_mult = 4; // The 1 in 4 always hits.
    score(&mut g, vec![Card::new(Value::Eight, Suit::Spade)]);
    assert!(matches!(g.consumables[..], [Consumable::Tarot(_)]));
    assert_eq!(notes(&mut g), [JokerNote::PlusTarot]);
}

#[test]
fn superposition_needs_an_ace_in_a_straight() {
    let mut g = game_with(vec![Jokers::Superposition(Default::default())]);
    score(
        &mut g,
        straight([
            Suit::Spade,
            Suit::Heart,
            Suit::Club,
            Suit::Diamond,
            Suit::Spade,
        ]),
    );
    assert_eq!(g.consumables.len(), 1);
}

#[test]
fn seance_makes_a_spectral_from_a_straight_flush() {
    let mut g = game_with(vec![Jokers::Seance(Default::default())]);
    score(&mut g, straight([Suit::Heart; 5]));
    assert!(matches!(g.consumables[..], [Consumable::Spectral(_)]));
}

#[test]
fn vagabond_makes_a_tarot_when_nearly_broke() {
    let mut g = game_with(vec![Jokers::Vagabond(Default::default())]);
    g.money = 4;
    score(&mut g, vec![Card::new(Value::Ace, Suit::Spade)]);
    assert_eq!(g.consumables.len(), 1);
    g.consumables.clear();
    g.money = 5;
    score(&mut g, vec![Card::new(Value::Ace, Suit::Spade)]);
    assert!(g.consumables.is_empty());
}

#[test]
fn loyalty_card_multiplies_every_sixth_hand() {
    let mut g = game_with(vec![Jokers::LoyaltyCard(Default::default())]);
    let ace = || vec![Card::new(Value::Ace, Suit::Spade)];
    for _ in 0..5 {
        assert_eq!(score(&mut g, ace()), 16);
        g.wear_seltzer();
    }
    assert_eq!(score(&mut g, ace()), 64, "the sixth hand");
}

#[test]
fn space_joker_levels_up_the_hand_before_it_scores() {
    let mut g = game_with(vec![Jokers::SpaceJoker(Default::default())]);
    g.prob_mult = 4;
    // High Card level 2: (15 + 11) × 2.
    assert_eq!(score(&mut g, vec![Card::new(Value::Ace, Suit::Spade)]), 52);
    assert_eq!(notes(&mut g), [JokerNote::LevelUp]);
}

#[test]
fn blind_select_makers_create_their_cards() {
    let mut g = game_with(vec![
        Jokers::Cartomancer(Default::default()),
        Jokers::MarbleJoker(Default::default()),
        Jokers::RiffRaff(Default::default()),
    ]);
    let deck = g.deck.len();
    g.jokers_set_blind(Blind::Small);
    assert_eq!(g.consumables.len(), 1, "Cartomancer");
    assert_eq!(g.deck.len(), deck + 1, "Marble Joker");
    assert!(g
        .deck
        .cards()
        .iter()
        .any(|c| c.enhancement == Some(Enhancement::Stone)));
    assert_eq!(g.jokers.len(), 5, "Riff-raff adds two Commons");
}

#[test]
fn burnt_joker_levels_up_the_first_discard_only() {
    let mut g = game_with(vec![Jokers::BurntJoker(Default::default())]);
    g.deal();
    g.discards_remaining = g.discards_per_round();
    let first = g.available.cards()[0];
    g.available.select_card(first).unwrap();
    g.discard_selected().unwrap();
    // A single card is at least a High Card; a pair would level Pair.
    let total: usize = HandRank::iter()
        .filter(|r| *r != HandRank::RoyalFlush)
        .map(|r| g.planetarium.level(r).level)
        .sum();
    assert_eq!(total, 12 + 1, "one hand gained a level");
    let second = g.available.cards()[0];
    g.available.select_card(second).unwrap();
    g.discard_selected().unwrap();
    let after: usize = HandRank::iter()
        .filter(|r| *r != HandRank::RoyalFlush)
        .map(|r| g.planetarium.level(r).level)
        .sum();
    assert_eq!(after, total, "only the first discard");
}

#[test]
fn diet_cola_leaves_a_double_tag_when_sold() {
    let mut g = game_with(vec![Jokers::DietCola(Default::default())]);
    g.stage = Stage::Shop();
    g.sell_joker(0).unwrap();
    assert!(g.tags.contains(&crate::tag::Tag::Double));
}

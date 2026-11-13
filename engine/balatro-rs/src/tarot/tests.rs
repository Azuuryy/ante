use super::*;
use crate::card::{Card, Suit, Value};
use crate::consumable::Consumable;
use crate::game::Game;
use crate::stage::{Blind, Stage};

fn game_in_blind() -> Game {
    Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    }
}

#[test]
fn test_target_counts() {
    assert_eq!(Tarot::Fool.min_targets(), 0);
    assert_eq!(Tarot::Fool.max_targets(), 0);
    assert_eq!(Tarot::Lovers.min_targets(), 1);
    assert_eq!(Tarot::Lovers.max_targets(), 1);
    assert_eq!(Tarot::Magician.min_targets(), 1);
    assert_eq!(Tarot::Magician.max_targets(), 2);
    assert_eq!(Tarot::Death.min_targets(), 2);
    assert_eq!(Tarot::Death.max_targets(), 2);
    assert_eq!(Tarot::Star.min_targets(), 1);
    assert_eq!(Tarot::Star.max_targets(), 3);
}

#[test]
fn test_requires_targets() {
    assert!(!Tarot::Fool.requires_targets());
    assert!(!Tarot::Hermit.requires_targets());
    assert!(Tarot::Lovers.requires_targets());
    assert!(Tarot::Death.requires_targets());
}

#[test]
fn test_strength_rank_wrap() {
    assert_eq!(Value::King.next(), Value::Ace);
    assert_eq!(Value::Ace.next(), Value::Two);
    assert_eq!(Value::Two.next(), Value::Three);
}

#[test]
fn test_hermit_cap() {
    let mut g = game_in_blind();
    g.money = 4;
    Tarot::Hermit.apply(&mut g).unwrap();
    assert_eq!(g.money, 8);

    let mut g = game_in_blind();
    g.money = 25;
    Tarot::Hermit.apply(&mut g).unwrap();
    assert_eq!(g.money, 45);

    let mut g = game_in_blind();
    g.money = 20;
    Tarot::Hermit.apply(&mut g).unwrap();
    assert_eq!(g.money, 40);
}

#[test]
fn test_death_copy_semantics() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let king = Card::new(Value::King, Suit::Diamond);
    let ace_id = ace.id;

    let mut g = game_in_blind();
    g.available.extend(vec![ace, king]);
    g.available.select_card(ace).unwrap();
    g.available.select_card(king).unwrap();
    g.deck.extend(vec![ace, king]);

    Tarot::Death.apply(&mut g).unwrap();

    let cards = g.available.cards();
    let left = cards.iter().find(|c| c.id == ace_id).unwrap();
    assert_eq!(left.value, Value::King);
    assert_eq!(left.suit, Suit::Diamond);
    assert_eq!(left.id, ace_id);
}

#[test]
fn test_enhancement_tarot() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let ace_id = ace.id;

    let mut g = game_in_blind();
    g.available.extend(vec![ace]);
    g.available.select_card(ace).unwrap();
    g.deck.extend(vec![ace]);

    Tarot::Justice.apply(&mut g).unwrap();

    let cards = g.available.cards();
    let card = cards.iter().find(|c| c.id == ace_id).unwrap();
    assert_eq!(card.enhancement, Some(Enhancement::Glass));

    let deck_cards = g.deck.cards();
    let deck_card = deck_cards.iter().find(|c| c.id == ace_id).unwrap();
    assert_eq!(deck_card.enhancement, Some(Enhancement::Glass));
}

#[test]
fn test_suit_tarot() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let ace_id = ace.id;

    let mut g = game_in_blind();
    g.available.extend(vec![ace]);
    g.available.select_card(ace).unwrap();
    g.deck.extend(vec![ace]);

    Tarot::Star.apply(&mut g).unwrap();

    let cards = g.available.cards();
    let card = cards.iter().find(|c| c.id == ace_id).unwrap();
    assert_eq!(card.suit, Suit::Diamond);
}

#[test]
fn test_hanged_man_destroys() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let king = Card::new(Value::King, Suit::Diamond);
    let ace_id = ace.id;

    let mut g = game_in_blind();
    g.available.extend(vec![ace, king]);
    g.available.select_card(ace).unwrap();
    g.deck.extend(vec![ace, king]);

    let deck_before = g.deck.cards().len();
    assert_eq!(g.available.cards().len(), 2);
    Tarot::HangedMan.apply(&mut g).unwrap();
    assert_eq!(g.available.cards().len(), 1);
    assert_eq!(g.deck.cards().len(), deck_before - 1);
    assert!(!g.deck.cards().iter().any(|c| c.id == ace_id));
}

#[test]
fn test_fool_copies_last_consumable() {
    use crate::planet::Planets;

    let mut g = game_in_blind();
    g.config.consumable_slots = 2;
    g.last_consumable_used = Some(Consumable::Planet(Planets::Mercury));

    Tarot::Fool.apply(&mut g).unwrap();
    assert_eq!(g.consumables.len(), 1);
    assert_eq!(g.consumables[0], Consumable::Planet(Planets::Mercury));
}

#[test]
fn test_fool_noop_when_no_last() {
    let mut g = game_in_blind();
    Tarot::Fool.apply(&mut g).unwrap();
    assert!(g.consumables.is_empty());
}

#[test]
fn test_strength_wraps_ace() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let ace_id = ace.id;

    let mut g = game_in_blind();
    g.available.extend(vec![ace]);
    g.available.select_card(ace).unwrap();
    g.deck.extend(vec![ace]);

    Tarot::Strength.apply(&mut g).unwrap();

    let cards = g.available.cards();
    let card = cards.iter().find(|c| c.id == ace_id).unwrap();
    assert_eq!(card.value, Value::Two);
}

#[test]
fn test_strength_ten_to_jack_becomes_face() {
    let ten = Card::new(Value::Ten, Suit::Heart);
    let ten_id = ten.id;
    assert!(!ten.is_face_card());

    let mut g = game_in_blind();
    g.available.extend(vec![ten]);
    g.available.select_card(ten).unwrap();
    g.deck.extend(vec![ten]);

    Tarot::Strength.apply(&mut g).unwrap();

    let cards = g.available.cards();
    let card = cards.iter().find(|c| c.id == ten_id).unwrap();
    assert_eq!(card.value, Value::Jack);
    assert!(card.is_face_card());
}

#[test]
fn test_strength_king_to_ace_loses_face() {
    let king = Card::new(Value::King, Suit::Heart);
    let king_id = king.id;
    assert!(king.is_face_card());

    let mut g = game_in_blind();
    g.available.extend(vec![king]);
    g.available.select_card(king).unwrap();
    g.deck.extend(vec![king]);

    Tarot::Strength.apply(&mut g).unwrap();

    let cards = g.available.cards();
    let card = cards.iter().find(|c| c.id == king_id).unwrap();
    assert_eq!(card.value, Value::Ace);
    assert!(!card.is_face_card());
}

#[test]
fn test_wheel_of_fortune_noop_without_jokers() {
    let mut g = game_in_blind();
    Tarot::WheelOfFortune.apply(&mut g).unwrap();
    assert!(g.jokers.is_empty());
}

#[test]
fn test_wheel_of_fortune_gates_edition_by_prob_roll() {
    use crate::joker::{Jokers, TheJoker};
    use balatro_types::Edition;

    let mut g = game_in_blind();
    g.jokers.push(Jokers::TheJoker(TheJoker::default()));

    let mut saw_base = false;
    let mut saw_edition = false;
    for _ in 0..200 {
        Tarot::WheelOfFortune.apply(&mut g).unwrap();

        assert_eq!(g.jokers.len(), 1, "joker count must not change");
        match g.jokers[0].edition() {
            Edition::Base => saw_base = true,
            Edition::Foil | Edition::Holographic | Edition::Polychrome => saw_edition = true,
            Edition::Negative => panic!("Wheel of Fortune must never apply Negative"),
        }
        g.jokers[0].set_edition(Edition::Base);
    }

    assert!(
        saw_base,
        "1-in-4 gate should sometimes not fire in 200 tries"
    );
    assert!(
        saw_edition,
        "1-in-4 gate should sometimes fire in 200 tries"
    );
}

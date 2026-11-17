//! Lot 3, batch 6: Blueprint, Brainstorm, Triboulet and Canio.

use super::*;

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

fn score(g: &mut Game, cards: Vec<Card>) -> usize {
    g.calc_score(SelectHand::new(cards).best_hand().unwrap())
}

fn ace() -> Vec<Card> {
    vec![Card::new(Value::Ace, Suit::Spade)]
}

#[test]
fn blueprint_copies_the_joker_to_its_right() {
    // The Joker twice: High Card Ace 16 chips × (1 + 4 + 4).
    let mut g = game_with(vec![
        Jokers::Blueprint(Default::default()),
        Jokers::TheJoker(Default::default()),
    ]);
    assert_eq!(score(&mut g, ace()), 16 * 9);
    // Moved to the right end, it copies nothing.
    g.jokers.swap(0, 1);
    assert_eq!(score(&mut g, ace()), 16 * 5);
}

#[test]
fn brainstorm_copies_the_leftmost_joker() {
    let mut g = game_with(vec![
        Jokers::TheJoker(Default::default()),
        Jokers::Brainstorm(Default::default()),
    ]);
    assert_eq!(score(&mut g, ace()), 16 * 9);
}

#[test]
fn a_copy_does_not_make_its_target_grow() {
    let mut g = game_with(vec![
        Jokers::Blueprint(Default::default()),
        Jokers::SpareTrousers(Default::default()),
    ]);
    let two_pair = vec![
        Card::new(Value::King, Suit::Spade),
        Card::new(Value::King, Suit::Heart),
        Card::new(Value::Two, Suit::Spade),
        Card::new(Value::Two, Suit::Heart),
    ];
    score(&mut g, two_pair);
    assert_eq!(g.jokers[1].state().counter, 2.0, "Spare Trousers grew once");
}

#[test]
fn copies_of_copies_stop() {
    // Blueprint → Brainstorm → leftmost is Blueprint again: no endless loop.
    let mut g = game_with(vec![
        Jokers::Blueprint(Default::default()),
        Jokers::Brainstorm(Default::default()),
    ]);
    assert_eq!(score(&mut g, ace()), 16);
}

#[test]
fn triboulet_doubles_kings_and_queens() {
    let mut g = game_with(vec![Jokers::Triboulet(Default::default())]);
    // Pair of Kings: (10 + 20) × 2, then X2 per King.
    let kings = vec![
        Card::new(Value::King, Suit::Spade),
        Card::new(Value::King, Suit::Heart),
    ];
    assert_eq!(score(&mut g, kings), 30 * 8);
}

#[test]
fn canio_grows_when_a_face_card_is_destroyed() {
    let mut g = game_with(vec![Jokers::Canio(Default::default())]);
    g.deal();
    let card = g.available.cards()[0];
    g.mutate_card(card.id, |c| c.value = Value::Jack);
    g.destroy_card(card.id);
    assert_eq!(g.jokers[0].state().counter, 1.0);
    assert_eq!(score(&mut g, ace()), 32);
}

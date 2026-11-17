//! Lot 3, batch 5b: Sixth Sense, DNA, Certificate, Showman, Astronomer, Invisible Joker.

use super::*;
use crate::consumable::Consumable;
use crate::pack::PackCategory;
use crate::planet::Planets;

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

fn play_one(g: &mut Game, value: Value) -> Card {
    g.deal();
    let card = g.available.cards()[0];
    g.mutate_card(card.id, move |c| c.value = value);
    g.available.select_card(card).unwrap();
    g.play_selected().unwrap();
    card
}

#[test]
fn sixth_sense_destroys_a_lone_first_six_for_a_spectral() {
    let mut g = game_with(vec![Jokers::SixthSense(Default::default())]);
    let six = play_one(&mut g, Value::Six);
    assert!(matches!(g.consumables[..], [Consumable::Spectral(_)]));
    assert!(
        !g.full_deck().iter().any(|c| c.id == six.id),
        "the 6 is gone"
    );
}

#[test]
fn dna_copies_a_lone_first_card_into_the_hand() {
    let mut g = game_with(vec![Jokers::Dna(Default::default())]);
    let total = g.full_deck().len();
    play_one(&mut g, Value::King);
    assert_eq!(g.full_deck().len(), total + 1);
}

#[test]
fn certificate_adds_a_sealed_card_when_the_round_starts() {
    let mut g = game_with(vec![Jokers::Certificate(Default::default())]);
    g.deal();
    let before = g.available.cards().len();
    g.certificates();
    let cards = g.available.cards();
    assert_eq!(cards.len(), before + 1);
    assert!(cards.last().unwrap().seal.is_some());
}

#[test]
fn astronomer_makes_planets_and_celestial_packs_free() {
    let g = game_with(vec![Jokers::Astronomer(Default::default())]);
    let planet = Consumable::Planet(Planets::Pluto);
    assert_eq!(g.shop.consumable_price(&planet, &g.vouchers), 0);
    let pack = crate::pack::Pack {
        category: PackCategory::Celestial,
        size: crate::pack::PackSize::Normal,
        contents: vec![],
    };
    assert_eq!(g.shop.pack_price(&pack, &g.vouchers), 0);
    let bare = game_with(vec![]);
    assert_eq!(bare.shop.consumable_price(&planet, &bare.vouchers), 3);
}

#[test]
fn invisible_joker_duplicates_a_joker_after_two_rounds() {
    let mut g = game_with(vec![
        Jokers::InvisibleJoker(Default::default()),
        Jokers::TheJoker(Default::default()),
    ]);
    g.stage = Stage::Shop();
    g.handle_round_end();
    g.handle_round_end();
    g.sell_joker(0).unwrap();
    assert_eq!(g.jokers.len(), 2);
    assert!(g.jokers.iter().all(|j| matches!(j, Jokers::TheJoker(_))));
}

#[test]
fn invisible_joker_sold_too_early_does_nothing() {
    let mut g = game_with(vec![
        Jokers::InvisibleJoker(Default::default()),
        Jokers::TheJoker(Default::default()),
    ]);
    g.stage = Stage::Shop();
    g.handle_round_end();
    g.sell_joker(0).unwrap();
    assert_eq!(g.jokers.len(), 1);
}

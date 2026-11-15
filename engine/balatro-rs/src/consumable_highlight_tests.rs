// Selection after a consumable, after the original Card:use_consumeable: enhancement,
// suit, Strength and Death Tarots and the seal Spectrals unhighlight the hand.
// Also checks that the action space offers the same consumable and pack moves as
// gen_actions.

use crate::action::Action;
use crate::consumable::Consumable;
use crate::game::Game;
use crate::pack::{OpenPackState, PackContent};
use crate::planet::Planets;
use crate::spectral::Spectral;
use crate::stage::{Blind, Stage};
use crate::tarot::Tarot;

fn blind_game() -> Game {
    let mut g = Game::default();
    g.start();
    g.handle_action(Action::SelectBlind(Blind::Small)).unwrap();
    g
}

fn select(g: &mut Game, count: usize) {
    for card in g.available.not_selected().into_iter().take(count) {
        g.handle_action(Action::SelectCard(card)).unwrap();
    }
}

fn space_actions(g: &Game) -> Vec<Action> {
    let space = g.gen_action_space();
    let mask = space.to_vec();
    (0..mask.len())
        .filter(|&i| mask[i] == 1)
        .map(|i| space.to_action(i, g).expect("unmasked index converts"))
        .collect()
}

fn open_pack(g: &mut Game, contents: Vec<PackContent>, picks: usize) {
    g.stage = Stage::PackOpen();
    g.open_pack = Some(OpenPackState {
        contents,
        picks_remaining: picks,
        description: String::new(),
    });
}

#[test]
fn lovers_used_in_a_blind_clears_the_selection() {
    let mut g = blind_game();
    select(&mut g, 1);
    g.consumables = vec![Consumable::Tarot(Tarot::Lovers)];
    g.handle_action(Action::UseConsumable(Consumable::Tarot(Tarot::Lovers)))
        .unwrap();
    assert!(g.available.selected().is_empty());
}

#[test]
fn talisman_used_in_a_blind_clears_the_selection() {
    let mut g = blind_game();
    select(&mut g, 1);
    g.consumables = vec![Consumable::Spectral(Spectral::Talisman)];
    g.handle_action(Action::UseConsumable(Consumable::Spectral(
        Spectral::Talisman,
    )))
    .unwrap();
    assert!(g.available.selected().is_empty());
}

#[test]
fn aura_used_in_a_blind_keeps_the_selection() {
    let mut g = blind_game();
    select(&mut g, 1);
    g.consumables = vec![Consumable::Spectral(Spectral::Aura)];
    g.handle_action(Action::UseConsumable(Consumable::Spectral(Spectral::Aura)))
        .unwrap();
    assert_eq!(g.available.selected().len(), 1);
}

#[test]
fn a_second_tarot_from_the_same_pack_starts_with_no_selection() {
    let mut g = blind_game();
    open_pack(
        &mut g,
        vec![
            PackContent::Tarot(Tarot::World),
            PackContent::Tarot(Tarot::Lovers),
        ],
        2,
    );
    select(&mut g, 3);
    g.handle_action(Action::PickPackCard(PackContent::Tarot(Tarot::World)))
        .unwrap();
    assert!(g.available.selected().is_empty());
    select(&mut g, 1);
    let lovers = Action::PickPackCard(PackContent::Tarot(Tarot::Lovers));
    assert!(space_actions(&g).contains(&lovers));
    g.handle_action(lovers).unwrap();
}

#[test]
fn slot_consumables_stay_usable_while_a_pack_is_open() {
    // `can_use_consumeable` only blocks while a hand is played or drawn, not in packs.
    let mut g = blind_game();
    g.consumables = vec![Consumable::Planet(Planets::Neptune)];
    open_pack(&mut g, vec![PackContent::Tarot(Tarot::Hermit)], 1);
    let neptune = Action::UseConsumable(Consumable::Planet(Planets::Neptune));
    assert!(g.gen_actions().any(|a| a == neptune));
    assert!(space_actions(&g).contains(&neptune));
    g.handle_action(neptune).unwrap();
    assert_eq!(g.stage, Stage::PackOpen());
}

#[test]
fn the_action_space_follows_the_consumable_use_rules() {
    let mut g = blind_game();
    g.consumables = vec![Consumable::Tarot(Tarot::Fool)];
    assert_eq!(g.last_consumable_used, None);
    assert!(!space_actions(&g)
        .iter()
        .any(|a| matches!(a, Action::UseConsumable(_))));
}

#[test]
fn the_action_space_offers_no_pack_card_without_room() {
    let mut g = blind_game();
    g.consumables = vec![Consumable::Planet(Planets::Mercury); g.consumable_slots()];
    open_pack(&mut g, vec![PackContent::Tarot(Tarot::Fool)], 1);
    assert!(!space_actions(&g)
        .iter()
        .any(|a| matches!(a, Action::PickPackCard(_))));
}

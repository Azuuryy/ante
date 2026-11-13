// Room for cards created by The Emperor and The High Priestess, from a slot, a pack or
// the shop, after the original can_use_consumeable and use_consumeable rules.

use crate::action::Action;
use crate::consumable::Consumable;
use crate::game::Game;
use crate::pack::{OpenPackState, PackContent};
use crate::planet::Planets;
use crate::stage::{Blind, Stage};
use crate::tarot::Tarot;

const EMPEROR: Consumable = Consumable::Tarot(Tarot::Emperor);
const MERCURY: Consumable = Consumable::Planet(Planets::Mercury);

fn shop_game() -> Game {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.money = 100;
    g
}

fn tarots(g: &Game) -> usize {
    g.consumables
        .iter()
        .filter(|c| matches!(c, Consumable::Tarot(_)))
        .count()
}

#[test]
fn emperor_used_from_full_slots_frees_its_own_slot() {
    let mut g = Game::default();
    g.start();
    g.handle_action(Action::SelectBlind(Blind::Small)).unwrap();
    g.consumables = vec![EMPEROR, MERCURY];
    g.handle_action(Action::UseConsumable(EMPEROR)).unwrap();
    assert_eq!(g.consumables.len(), 2);
    assert_eq!(tarots(&g), 1);
}

fn open_arcana(g: &mut Game) {
    g.stage = Stage::PackOpen();
    g.open_pack = Some(OpenPackState {
        contents: vec![PackContent::Tarot(Tarot::Emperor)],
        picks_remaining: 1,
        description: String::new(),
    });
}

#[test]
fn emperor_in_a_pack_needs_a_free_slot() {
    let mut g = shop_game();
    g.consumables = vec![MERCURY; g.consumable_slots()];
    open_arcana(&mut g);
    let pick = Action::PickPackCard(PackContent::Tarot(Tarot::Emperor));
    assert!(!g.gen_actions().any(|a| a == pick));
    assert!(g.handle_action(pick).is_err());
    assert_eq!(g.open_pack.as_ref().unwrap().contents.len(), 1);
}

#[test]
fn emperor_in_a_pack_fills_the_free_slots() {
    let mut g = shop_game();
    g.consumables = vec![MERCURY];
    open_arcana(&mut g);
    g.handle_action(Action::PickPackCard(PackContent::Tarot(Tarot::Emperor)))
        .unwrap();
    assert_eq!(g.consumables.len(), g.consumable_slots());
    assert_eq!(tarots(&g), g.consumable_slots() - 1);
}

#[test]
fn buy_and_use_skips_the_slot_and_pays() {
    let mut g = shop_game();
    g.shop.consumables = vec![MERCURY];
    g.consumables = vec![EMPEROR; g.consumable_slots()];
    let before = g.money;
    assert!(g.can_buy_and_use(&MERCURY));
    g.buy_and_use_consumable(MERCURY).unwrap();
    assert!(g.money < before);
    assert!(g.shop.consumables.is_empty());
    assert_eq!(g.consumables.len(), g.consumable_slots());
}

#[test]
fn buy_and_use_emperor_needs_room_like_the_original() {
    let mut g = shop_game();
    g.shop.consumables = vec![EMPEROR];
    g.consumables = vec![MERCURY; g.consumable_slots()];
    assert!(!g.can_buy_and_use(&EMPEROR));
    assert!(g.buy_and_use_consumable(EMPEROR).is_err());
    g.consumables.pop();
    g.buy_and_use_consumable(EMPEROR).unwrap();
    assert_eq!(g.consumables.len(), g.consumable_slots());
}

#[test]
fn targeted_tarots_cannot_be_bought_and_used() {
    let mut g = shop_game();
    let lovers = Consumable::Tarot(Tarot::Lovers);
    g.shop.consumables = vec![lovers];
    assert!(!g.can_buy_and_use(&lovers));
}

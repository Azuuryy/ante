// Tests de disponibilité des consommables, d'après les règles d'origine
// (Card:can_use_consumeable), paraphrasées. Les tests marqués FAIL attendu
// décrivent le comportement voulu et doivent échouer avant le correctif.

use crate::action::Action;
use crate::card::Edition;
use crate::consumable::Consumable;
use crate::error::GameError;
use crate::game::Game;
use crate::joker::{HangingChad, Jokers};
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

fn offered(g: &Game, c: Consumable) -> bool {
    g.gen_actions().any(|a| a == Action::UseConsumable(c))
}

fn fill_joker_slots(g: &mut Game) {
    while g.jokers.len() < g.joker_slots() {
        g.jokers.push(Jokers::HangingChad(HangingChad::default()));
    }
}

// ---------- Familiar : zéro cible, mais utilisable quelle que soit la sélection ----------

#[test]
fn familiar_offered_in_blind_without_selection() {
    let mut g = blind_game();
    g.consumables = vec![Consumable::Spectral(Spectral::Familiar)];
    assert!(g.available.cards().len() > 1);
    assert!(offered(&g, Consumable::Spectral(Spectral::Familiar)));
}

#[test]
fn familiar_offered_in_blind_with_a_card_selected() {
    // FAIL attendu : le filtre actuel exige selected_count == 0.
    let mut g = blind_game();
    let card = g.available.cards()[0];
    g.handle_action(Action::SelectCard(card)).unwrap();
    g.consumables = vec![Consumable::Spectral(Spectral::Familiar)];
    assert!(offered(&g, Consumable::Spectral(Spectral::Familiar)));
}

#[test]
fn familiar_use_with_a_card_selected_applies() {
    // FAIL attendu : use_consumable refuse une sélection non nulle.
    let mut g = blind_game();
    let card = g.available.cards()[0];
    g.handle_action(Action::SelectCard(card)).unwrap();
    g.consumables = vec![Consumable::Spectral(Spectral::Familiar)];
    let res = g.handle_action(Action::UseConsumable(Consumable::Spectral(
        Spectral::Familiar,
    )));
    assert!(
        res.is_ok(),
        "Familiar doit s'appliquer avec une carte sélectionnée"
    );
    assert!(g.consumables.is_empty());
}

#[test]
fn immolate_offered_in_blind_with_full_hand() {
    // Immolate est aussi à zéro cible et touche la main.
    let mut g = blind_game();
    g.consumables = vec![Consumable::Spectral(Spectral::Immolate)];
    assert!(offered(&g, Consumable::Spectral(Spectral::Immolate)));
}

#[test]
fn hand_touching_spectral_not_offered_in_shop() {
    // Garde de non-régression : le blocage en Shop doit rester (main vide).
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.consumables = vec![
        Consumable::Spectral(Spectral::Familiar),
        Consumable::Spectral(Spectral::Immolate),
    ];
    assert!(!offered(&g, Consumable::Spectral(Spectral::Familiar)));
    assert!(!offered(&g, Consumable::Spectral(Spectral::Immolate)));
}

#[test]
fn hermit_still_offered_in_shop() {
    // Garde de non-régression : sans cible, utilisable hors Blind.
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.consumables = vec![Consumable::Tarot(Tarot::Hermit)];
    assert!(offered(&g, Consumable::Tarot(Tarot::Hermit)));
}

// ---------- Wraith / Soul / Judgement : place de joker requise ----------

#[test]
fn wraith_refused_when_joker_slots_full() {
    // FAIL attendu : offert, et money passe à 0 même sans joker ajouté.
    let mut g = blind_game();
    fill_joker_slots(&mut g);
    g.money = 10;
    g.consumables = vec![Consumable::Spectral(Spectral::Wraith)];
    assert!(!offered(&g, Consumable::Spectral(Spectral::Wraith)));
    let res = g.handle_action(Action::UseConsumable(Consumable::Spectral(
        Spectral::Wraith,
    )));
    assert!(matches!(res, Err(GameError::InvalidAction)));
    assert_eq!(g.consumables, vec![Consumable::Spectral(Spectral::Wraith)]);
    assert_eq!(g.money, 10);
}

#[test]
fn wraith_creates_joker_and_zeroes_money_when_room() {
    // PASS attendu : comportement d'origine conservé.
    let mut g = blind_game();
    g.money = 10;
    let before = g.jokers.len();
    g.consumables = vec![Consumable::Spectral(Spectral::Wraith)];
    g.handle_action(Action::UseConsumable(Consumable::Spectral(
        Spectral::Wraith,
    )))
    .unwrap();
    assert_eq!(g.jokers.len(), before + 1);
    assert_eq!(g.money, 0);
}

#[test]
fn soul_refused_when_joker_slots_full() {
    // FAIL attendu.
    let mut g = blind_game();
    fill_joker_slots(&mut g);
    g.consumables = vec![Consumable::Spectral(Spectral::Soul)];
    assert!(!offered(&g, Consumable::Spectral(Spectral::Soul)));
}

#[test]
fn judgement_refused_when_joker_slots_full() {
    // FAIL attendu : offert, puis carte consommée sans rien créer.
    let mut g = blind_game();
    fill_joker_slots(&mut g);
    g.consumables = vec![Consumable::Tarot(Tarot::Judgement)];
    assert!(!offered(&g, Consumable::Tarot(Tarot::Judgement)));
    let res = g.handle_action(Action::UseConsumable(Consumable::Tarot(Tarot::Judgement)));
    assert!(matches!(res, Err(GameError::InvalidAction)));
    assert_eq!(g.consumables, vec![Consumable::Tarot(Tarot::Judgement)]);
}

// ---------- Ankh / Hex / Ectoplasm : au moins un joker ----------

#[test]
fn ankh_refused_without_jokers() {
    // FAIL attendu : offert sans joker, carte consommée sans effet.
    let mut g = blind_game();
    g.jokers.clear();
    g.consumables = vec![Consumable::Spectral(Spectral::Ankh)];
    assert!(!offered(&g, Consumable::Spectral(Spectral::Ankh)));
}

#[test]
fn ankh_duplicates_a_joker_with_a_joker_present() {
    // PASS attendu : copie d'un joker, les autres sont détruits.
    let mut g = blind_game();
    g.jokers = vec![Jokers::HangingChad(HangingChad::default())];
    g.consumables = vec![Consumable::Spectral(Spectral::Ankh)];
    g.handle_action(Action::UseConsumable(Consumable::Spectral(Spectral::Ankh)))
        .unwrap();
    assert_eq!(g.jokers.len(), 2);
}

#[test]
fn ectoplasm_refused_when_every_joker_already_has_an_edition() {
    // FAIL attendu : l'original ne cible que les jokers sans édition.
    let mut g = blind_game();
    let mut j = Jokers::HangingChad(HangingChad::default());
    j.set_edition(Edition::Polychrome);
    g.jokers = vec![j];
    g.consumables = vec![Consumable::Spectral(Spectral::Ectoplasm)];
    assert!(!offered(&g, Consumable::Spectral(Spectral::Ectoplasm)));
}

// ---------- The Fool : dépend du dernier consommable utilisé ----------

#[test]
fn the_fool_refused_without_last_consumable() {
    // FAIL attendu : offert sans last_consumable_used, carte perdue.
    let mut g = blind_game();
    g.last_consumable_used = None;
    g.consumables = vec![Consumable::Tarot(Tarot::Fool)];
    assert!(!offered(&g, Consumable::Tarot(Tarot::Fool)));
}

#[test]
fn the_fool_refused_when_last_consumable_is_the_fool() {
    // FAIL attendu : The Fool ne se copie jamais lui-même.
    let mut g = blind_game();
    g.last_consumable_used = Some(Consumable::Tarot(Tarot::Fool));
    g.consumables = vec![Consumable::Tarot(Tarot::Fool)];
    assert!(!offered(&g, Consumable::Tarot(Tarot::Fool)));
}

#[test]
fn the_fool_copies_last_planet_when_room() {
    // PASS attendu.
    let mut g = blind_game();
    g.last_consumable_used = Some(Consumable::Planet(Planets::Mercury));
    g.consumables = vec![Consumable::Tarot(Tarot::Fool)];
    assert!(offered(&g, Consumable::Tarot(Tarot::Fool)));
    g.handle_action(Action::UseConsumable(Consumable::Tarot(Tarot::Fool)))
        .unwrap();
    assert!(g
        .consumables
        .contains(&Consumable::Planet(Planets::Mercury)));
}

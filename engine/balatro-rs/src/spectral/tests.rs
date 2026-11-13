use super::*;
use crate::card::{Card, Enhancement, Suit, Value};
use crate::joker::{jokers_by_rarity, mint_joker_id, Jokers};
use crate::stage::{Blind, Stage};
use std::collections::HashSet;

fn game_in_blind() -> Game {
    Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    }
}

fn minted_joker(rarity: Rarity, index: usize) -> Jokers {
    let mut j = jokers_by_rarity(rarity)[index].clone();
    j.set_instance_id(mint_joker_id());
    j
}

#[test]
fn test_random_spectral_excludes_rare() {
    let mut rng = rand::thread_rng();
    for _ in 0..500 {
        let s = random_spectral(&mut rng);
        assert!(!s.is_rare(), "{s:?} should never be drawn as a normal pick");
    }
}

#[test]
fn test_talisman_adds_gold_seal() {
    let card = Card::new(Value::Ace, Suit::Heart);
    let mut g = game_in_blind();
    g.available.extend(vec![card]);
    g.available.select_card(card).unwrap();
    Spectral::Talisman.apply(&mut g).unwrap();
    let updated = g
        .available
        .cards()
        .into_iter()
        .find(|c| c.id == card.id)
        .unwrap();
    assert_eq!(updated.seal, Some(Seal::Gold));
}

#[test]
fn test_deja_vu_adds_red_seal() {
    let card = Card::new(Value::Ace, Suit::Heart);
    let mut g = game_in_blind();
    g.available.extend(vec![card]);
    g.available.select_card(card).unwrap();
    Spectral::DejaVu.apply(&mut g).unwrap();
    let updated = g
        .available
        .cards()
        .into_iter()
        .find(|c| c.id == card.id)
        .unwrap();
    assert_eq!(updated.seal, Some(Seal::Red));
}

#[test]
fn test_trance_adds_blue_seal() {
    let card = Card::new(Value::Ace, Suit::Heart);
    let mut g = game_in_blind();
    g.available.extend(vec![card]);
    g.available.select_card(card).unwrap();
    Spectral::Trance.apply(&mut g).unwrap();
    let updated = g
        .available
        .cards()
        .into_iter()
        .find(|c| c.id == card.id)
        .unwrap();
    assert_eq!(updated.seal, Some(Seal::Blue));
}

#[test]
fn test_medium_adds_purple_seal() {
    let card = Card::new(Value::Ace, Suit::Heart);
    let mut g = game_in_blind();
    g.available.extend(vec![card]);
    g.available.select_card(card).unwrap();
    Spectral::Medium.apply(&mut g).unwrap();
    let updated = g
        .available
        .cards()
        .into_iter()
        .find(|c| c.id == card.id)
        .unwrap();
    assert_eq!(updated.seal, Some(Seal::Purple));
}

#[test]
fn test_aura_adds_foil_holo_or_polychrome() {
    let mut seen = HashSet::new();
    for _ in 0..60 {
        let card = Card::new(Value::Ace, Suit::Heart);
        let mut g = game_in_blind();
        g.available.extend(vec![card]);
        g.available.select_card(card).unwrap();
        Spectral::Aura.apply(&mut g).unwrap();
        let updated = g
            .available
            .cards()
            .into_iter()
            .find(|c| c.id == card.id)
            .unwrap();
        assert!(matches!(
            updated.edition,
            Edition::Foil | Edition::Holographic | Edition::Polychrome
        ));
        seen.insert(updated.edition);
    }
    assert!(
        seen.len() > 1,
        "expected multiple edition outcomes across 60 draws"
    );
}

#[test]
fn test_cryptid_creates_two_matching_copies() {
    let mut og = Card::new(Value::King, Suit::Spade);
    og.enhancement = Some(Enhancement::Glass);
    og.edition = Edition::Foil;
    og.seal = Some(Seal::Gold);
    let mut g = game_in_blind();
    g.available.extend(vec![og]);
    g.available.select_card(og).unwrap();

    Spectral::Cryptid.apply(&mut g).unwrap();

    let cards = g.available.cards();
    assert_eq!(cards.len(), 3);
    let original = cards.iter().find(|c| c.id == og.id).unwrap();
    assert_eq!(*original, og);

    let copies: Vec<_> = cards.iter().filter(|c| c.id != og.id).collect();
    assert_eq!(copies.len(), 2);
    assert_ne!(copies[0].id, copies[1].id);
    for copy in copies {
        assert_eq!(copy.value, og.value);
        assert_eq!(copy.suit, og.suit);
        assert_eq!(copy.enhancement, og.enhancement);
        assert_eq!(copy.edition, og.edition);
        assert_eq!(copy.seal, og.seal);
    }
}

#[test]
fn test_single_target_spectrals_reject_bad_selection_counts() {
    let variants = [
        Spectral::Talisman,
        Spectral::DejaVu,
        Spectral::Trance,
        Spectral::Medium,
        Spectral::Aura,
        Spectral::Cryptid,
    ];
    for s in variants {
        let mut none_selected = game_in_blind();
        none_selected
            .available
            .extend(vec![Card::new(Value::Ace, Suit::Heart)]);
        assert!(
            matches!(s.apply(&mut none_selected), Err(GameError::InvalidAction)),
            "{s:?} should reject 0 selected cards"
        );

        let mut two_selected = game_in_blind();
        let a = Card::new(Value::Ace, Suit::Heart);
        let k = Card::new(Value::King, Suit::Diamond);
        two_selected.available.extend(vec![a, k]);
        two_selected.available.select_card(a).unwrap();
        two_selected.available.select_card(k).unwrap();
        assert!(
            matches!(s.apply(&mut two_selected), Err(GameError::InvalidAction)),
            "{s:?} should reject 2 selected cards"
        );
    }
}

#[test]
fn test_familiar_destroys_one_adds_three_enhanced_face_cards() {
    let mut g = game_in_blind();
    g.available.extend(vec![Card::new(Value::Two, Suit::Heart)]);
    Spectral::Familiar.apply(&mut g).unwrap();
    let cards = g.available.cards();
    assert_eq!(cards.len(), 3);
    for c in cards {
        assert!(c.enhancement.is_some());
        assert!(matches!(c.value, Value::King | Value::Queen | Value::Jack));
    }
}

#[test]
fn test_grim_destroys_one_adds_two_enhanced_aces() {
    let mut g = game_in_blind();
    g.available.extend(vec![Card::new(Value::Two, Suit::Heart)]);
    Spectral::Grim.apply(&mut g).unwrap();
    let cards = g.available.cards();
    assert_eq!(cards.len(), 2);
    for c in cards {
        assert!(c.enhancement.is_some());
        assert_eq!(c.value, Value::Ace);
    }
}

#[test]
fn test_incantation_destroys_one_adds_four_enhanced_numbered_cards() {
    let mut g = game_in_blind();
    g.available
        .extend(vec![Card::new(Value::King, Suit::Heart)]);
    Spectral::Incantation.apply(&mut g).unwrap();
    let cards = g.available.cards();
    assert_eq!(cards.len(), 4);
    for c in cards {
        assert!(c.enhancement.is_some());
        assert!(matches!(
            c.value,
            Value::Two
                | Value::Three
                | Value::Four
                | Value::Five
                | Value::Six
                | Value::Seven
                | Value::Eight
                | Value::Nine
                | Value::Ten
        ));
    }
}

#[test]
fn test_sigil_converts_hand_to_single_suit() {
    let mut g = game_in_blind();
    g.available.extend(vec![
        Card::new(Value::Two, Suit::Heart),
        Card::new(Value::Three, Suit::Spade),
        Card::new(Value::Four, Suit::Diamond),
    ]);
    Spectral::Sigil.apply(&mut g).unwrap();
    let suits: HashSet<_> = g.available.cards().into_iter().map(|c| c.suit).collect();
    assert_eq!(suits.len(), 1);
}

#[test]
fn test_ouija_converts_hand_to_single_rank_and_shrinks_hand_size() {
    let mut g = game_in_blind();
    let available_before = g.config.available;
    g.available.extend(vec![
        Card::new(Value::Two, Suit::Heart),
        Card::new(Value::Three, Suit::Spade),
        Card::new(Value::Four, Suit::Diamond),
    ]);
    Spectral::Ouija.apply(&mut g).unwrap();
    let values: HashSet<_> = g.available.cards().into_iter().map(|c| c.value).collect();
    assert_eq!(values.len(), 1);
    assert_eq!(g.config.available, available_before - 1);
}

#[test]
fn test_immolate_destroys_up_to_five_and_gains_money() {
    let mut g = game_in_blind();
    g.available
        .extend((0..7).map(|_| Card::new(Value::Two, Suit::Heart)).collect());
    let money_before = g.money;
    Spectral::Immolate.apply(&mut g).unwrap();
    assert_eq!(g.available.cards().len(), 2);
    assert_eq!(g.money, money_before + 20);
}

#[test]
fn test_immolate_destroys_fewer_when_hand_is_smaller() {
    let mut g = game_in_blind();
    g.available.extend(vec![
        Card::new(Value::Two, Suit::Heart),
        Card::new(Value::Three, Suit::Spade),
    ]);
    let money_before = g.money;
    Spectral::Immolate.apply(&mut g).unwrap();
    assert_eq!(g.available.cards().len(), 0);
    assert_eq!(g.money, money_before + 20);
}

#[test]
fn test_black_hole_upgrades_every_hand_once_including_shared_royal_flush_slot() {
    let mut g = game_in_blind();
    let before: Vec<_> = HandRank::iter()
        .map(|r| (r, g.planetarium.level(r).level))
        .collect();
    Spectral::BlackHole.apply(&mut g).unwrap();
    for (rank, level_before) in before {
        assert_eq!(
            g.planetarium.level(rank).level,
            level_before + 1,
            "{rank:?} should level up by exactly 1"
        );
    }
}

#[test]
fn test_wraith_adds_one_rare_joker_and_zeroes_money() {
    let mut g = game_in_blind();
    g.money = 50;
    let jokers_before = g.jokers.len();
    Spectral::Wraith.apply(&mut g).unwrap();
    assert_eq!(g.jokers.len(), jokers_before + 1);
    assert_eq!(g.jokers.last().unwrap().rarity(), Rarity::Rare);
    assert_eq!(g.money, 0);
}

#[test]
fn test_wraith_noop_when_joker_slots_full() {
    let mut g = game_in_blind();
    g.jokers = vec![minted_joker(Rarity::Common, 0); g.config.joker_slots];
    g.money = 50;
    Spectral::Wraith.apply(&mut g).unwrap();
    assert_eq!(g.jokers.len(), g.config.joker_slots);
    assert_eq!(g.money, 0);
}

#[test]
fn test_soul_adds_one_legendary_joker_when_there_is_room() {
    let mut g = game_in_blind();
    let jokers_before = g.jokers.len();
    Spectral::Soul.apply(&mut g).unwrap();
    assert_eq!(g.jokers.len(), jokers_before + 1);
    assert_eq!(g.jokers.last().unwrap().rarity(), Rarity::Legendary);
}

#[test]
fn test_soul_noop_when_joker_slots_full() {
    let mut g = game_in_blind();
    g.jokers = vec![minted_joker(Rarity::Common, 0); g.config.joker_slots];
    Spectral::Soul.apply(&mut g).unwrap();
    assert_eq!(g.jokers.len(), g.config.joker_slots);
}

#[test]
fn test_ectoplasm_negatives_exactly_one_joker_and_shrinks_hand_size() {
    let mut g = game_in_blind();
    let available_before = g.config.available;
    g.jokers = vec![
        minted_joker(Rarity::Common, 0),
        minted_joker(Rarity::Common, 1),
        minted_joker(Rarity::Common, 2),
    ];
    Spectral::Ectoplasm.apply(&mut g).unwrap();
    let negative_count = g
        .jokers
        .iter()
        .filter(|j| j.edition() == Edition::Negative)
        .count();
    assert_eq!(negative_count, 1);
    assert_eq!(g.config.available, available_before - 1);
}

#[test]
fn test_ectoplasm_noop_when_no_jokers() {
    let mut g = game_in_blind();
    let available_before = g.config.available;
    Spectral::Ectoplasm.apply(&mut g).unwrap();
    assert!(g.jokers.is_empty());
    assert_eq!(g.config.available, available_before);
}

#[test]
fn test_ankh_copies_the_only_joker_and_strips_negative_edition() {
    let mut j = minted_joker(Rarity::Common, 0);
    j.set_edition(Edition::Negative);
    let mut g = game_in_blind();
    g.jokers = vec![j.clone()];

    Spectral::Ankh.apply(&mut g).unwrap();

    assert_eq!(g.jokers.len(), 2);
    let original = g
        .jokers
        .iter()
        .find(|k| k.instance_id() == j.instance_id())
        .unwrap();
    assert_eq!(original.edition(), Edition::Negative);
    let clone = g
        .jokers
        .iter()
        .find(|k| k.instance_id() != j.instance_id())
        .unwrap();
    assert_eq!(clone.edition(), Edition::Base);
}

#[test]
fn test_ankh_copy_keeps_non_negative_edition() {
    let mut j = minted_joker(Rarity::Common, 0);
    j.set_edition(Edition::Foil);
    let mut g = game_in_blind();
    g.jokers = vec![j.clone()];

    Spectral::Ankh.apply(&mut g).unwrap();

    assert_eq!(g.jokers.len(), 2);
    let clone = g
        .jokers
        .iter()
        .find(|k| k.instance_id() != j.instance_id())
        .unwrap();
    assert_eq!(clone.edition(), Edition::Foil);
}

#[test]
fn test_ankh_noop_when_no_jokers() {
    let mut g = game_in_blind();
    Spectral::Ankh.apply(&mut g).unwrap();
    assert!(g.jokers.is_empty());
}

#[test]
fn test_hex_mutates_the_only_joker_in_place() {
    let j = minted_joker(Rarity::Common, 0);
    let mut g = game_in_blind();
    g.jokers = vec![j.clone()];

    Spectral::Hex.apply(&mut g).unwrap();

    assert_eq!(g.jokers.len(), 1);
    assert_eq!(g.jokers[0].instance_id(), j.instance_id());
    assert_eq!(g.jokers[0].edition(), Edition::Polychrome);
}

#[test]
fn test_hex_destroys_other_jokers() {
    let j1 = minted_joker(Rarity::Common, 0);
    let j2 = minted_joker(Rarity::Common, 1);
    let j3 = minted_joker(Rarity::Common, 2);
    let mut g = game_in_blind();
    g.jokers = vec![j1, j2, j3];

    Spectral::Hex.apply(&mut g).unwrap();

    assert_eq!(g.jokers.len(), 1);
    assert_eq!(g.jokers[0].edition(), Edition::Polychrome);
}

#[test]
fn test_hex_noop_when_no_jokers() {
    let mut g = game_in_blind();
    Spectral::Hex.apply(&mut g).unwrap();
    assert!(g.jokers.is_empty());
}

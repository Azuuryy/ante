use super::*;

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

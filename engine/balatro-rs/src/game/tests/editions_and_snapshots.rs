use super::*;

#[test]
fn test_sock_and_buskin_retriggers_only_face_cards() {
    use crate::joker::*;
    // Two Pair (Jacks + Fours): SockAndBuskin retriggers each Jack, not the Fours.
    let cards = vec![
        Card::new(Value::Jack, Suit::Heart),
        Card::new(Value::Jack, Suit::Diamond),
        Card::new(Value::Four, Suit::Spade),
        Card::new(Value::Four, Suit::Club),
    ];

    let mut base_game = Game::default();
    let base_hand = SelectHand::new(cards.clone()).best_hand().unwrap();
    let base = base_game.calc_score(base_hand);

    let mut g = Game::default();
    g.jokers
        .push(Jokers::SockAndBuskin(SockAndBuskin::default()));
    let jokers = g.jokers.clone();
    g.effect_registry.register_jokers(jokers, &g.clone());
    let hand = SelectHand::new(cards).best_hand().unwrap();
    // Each Jack retriggers once more: +10 chips * 2 cards = +20 chips,
    // scaled by TwoPair's X2 mult -> +40 final score.
    assert_eq!(g.calc_score(hand), base + 40);
}

#[test]
fn test_mime_retriggers_held_steel() {
    use crate::joker::*;
    // Same math as test_seal_red_retrigger_steel_held (120), Mime instead of Red Seal:
    // mult 2 -> 3 (floor(2*1.5)) -> 4 (floor(3*1.5)); score = 30 * 4 = 120
    let mut g = Game::default();
    g.jokers.push(Jokers::Mime(Mime::default()));
    let jokers = g.jokers.clone();
    g.effect_registry.register_jokers(jokers, &g.clone());
    let king1 = Card::new(Value::King, Suit::Heart);
    let king2 = Card::new(Value::King, Suit::Diamond);
    let mut steel_king = Card::new(Value::King, Suit::Spade);
    steel_king.enhancement = Some(Enhancement::Steel);
    g.available.extend(vec![steel_king]);
    let hand = SelectHand::new(vec![king1, king2]).best_hand().unwrap();
    // Steel held twice: 2 * 1.5 * 1.5 = 4.5; (10 + 20) * 4.5 = 135
    assert_eq!(g.calc_score(hand), 135);
}

#[test]
fn test_seal_purple_discard_creates_tarot() {
    // Purple Seal: creates a Tarot card when this card is discarded.
    let mut g = Game::default();
    g.deal();
    let card = g.available.cards()[0];
    g.mutate_card(card.id, |c| c.seal = Some(Seal::Purple));
    let card = g.available.cards()[0];
    g.select_card(card).expect("can select card");

    assert_eq!(g.consumables.len(), 0);
    g.discard_selected().expect("can discard");
    assert_eq!(g.consumables.len(), 1);
    assert!(matches!(g.consumables[0], Consumable::Tarot(_)));
}

#[test]
fn test_seal_purple_discard_respects_slots() {
    let mut g = Game::default();
    g.config.consumable_slots = 0;
    g.deal();
    let card = g.available.cards()[0];
    g.mutate_card(card.id, |c| c.seal = Some(Seal::Purple));
    let card = g.available.cards()[0];
    g.select_card(card).expect("can select card");

    g.discard_selected().expect("can discard");
    assert_eq!(g.consumables.len(), 0);
}

#[test]
fn test_seal_blue_round_end_creates_planet() {
    // Blue Seal: creates a Planet card if held in hand when the round ends.
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Blind(Blind::Small);
    g.blind = Some(Blind::Small);
    g.deal();

    let held_card = g.available.cards()[7];
    g.mutate_card(held_card.id, |c| c.seal = Some(Seal::Blue));
    for card in g.available.cards()[0..5].to_vec() {
        g.available.select_card(card).expect("can select card");
    }
    assert_eq!(g.available.selected().len(), 5);

    g.score += g.required_score();
    assert_eq!(g.consumables.len(), 0);
    g.play_selected().expect("can play selected");

    assert_eq!(g.consumables.len(), 1);
    assert!(matches!(g.consumables[0], Consumable::Planet(_)));
}

#[test]
fn test_seal_blue_does_not_fire_before_blind_cleared() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Blind(Blind::Small);
    g.blind = Some(Blind::Small);
    g.deal();

    let held_card = g.available.cards()[7];
    g.mutate_card(held_card.id, |c| c.seal = Some(Seal::Blue));
    for card in g.available.cards()[0..5].to_vec() {
        g.available.select_card(card).expect("can select card");
    }
    // score not bumped past required_score() - blind stays open
    g.play_selected().expect("can play selected");

    assert_eq!(g.consumables.len(), 0);
}

#[test]
fn test_edition_foil() {
    // High card Ace with Foil: (5 + 11 + 50) * 1 = 66
    let mut g = Game::default();
    let mut ace = Card::new(Value::Ace, Suit::Heart);
    ace.edition = Edition::Foil;
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 66);
}

#[test]
fn test_edition_holographic() {
    // High card Ace with Holographic: (5 + 11) * (1 + 10) = 176
    let mut g = Game::default();
    let mut ace = Card::new(Value::Ace, Suit::Heart);
    ace.edition = Edition::Holographic;
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 176);
}

#[test]
fn test_edition_polychrome() {
    // Pair of Kings, second King has Polychrome
    // Pair level 1: chips=10, mult=2
    // king1: chips += 10; king2 (poly): chips += 10, mult = floor(2*1.5) = 3
    // score = (10+10+10) * 3 = 90
    let mut g = Game::default();
    let king = Card::new(Value::King, Suit::Heart);
    let mut poly_king = Card::new(Value::King, Suit::Diamond);
    poly_king.edition = Edition::Polychrome;
    let hand = SelectHand::new(vec![king, poly_king]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 90);
}

#[test]
fn test_joker_edition_foil() {
    use crate::joker::Jokers;
    use balatro_types::joker::TheJoker;
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();

    // TheJoker Base: (5+11)*(1+4) = 80
    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    let mut j = Jokers::TheJoker(TheJoker::default());
    j.set_edition(Edition::Foil);
    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    // Foil fires before TheJoker: chips += 50, then +4 mult
    // (5+11+50) * (1+4) = 66 * 5 = 330
    assert_eq!(g.calc_score(hand), 330);
}

#[test]
fn test_joker_edition_holographic() {
    use crate::joker::Jokers;
    use balatro_types::joker::TheJoker;
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    let mut j = Jokers::TheJoker(TheJoker::default());
    j.set_edition(Edition::Holographic);
    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    // Holo fires before TheJoker: mult += 10, then +4 mult
    // (5+11) * (1+10+4) = 16 * 15 = 240
    assert_eq!(g.calc_score(hand), 240);
}

#[test]
fn test_joker_edition_polychrome() {
    use crate::joker::Jokers;
    use balatro_types::joker::TheJoker;
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    let mut j = Jokers::TheJoker(TheJoker::default());
    j.set_edition(Edition::Polychrome);
    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    // TheJoker fires (+4 mult), then Polychrome x1.5: (1+4)*1.5 = 7.5
    // (5+11) * 7.5 = 120
    assert_eq!(g.calc_score(hand), 120);
}

#[test]
fn test_joker_edition_negative_buy_increases_slots() {
    use crate::joker::Jokers;
    use balatro_types::joker::TheJoker;
    let mut g = Game::default();
    let slots_before = g.config.joker_slots;

    g.money += 1000;
    g.stage = Stage::Shop();
    let mut j = Jokers::TheJoker(TheJoker::default());
    j.set_edition(Edition::Negative);
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();

    assert_eq!(g.config.joker_slots, slots_before + 1);
}

#[test]
fn test_joker_edition_negative_sell_decreases_slots() {
    use crate::joker::Jokers;
    use balatro_types::joker::TheJoker;
    let mut g = Game::default();
    let slots_before = g.config.joker_slots;

    g.money += 1000;
    g.stage = Stage::Shop();
    let mut j = Jokers::TheJoker(TheJoker::default());
    j.set_edition(Edition::Negative);
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    assert_eq!(g.config.joker_slots, slots_before + 1);

    g.sell_joker(0).unwrap();
    assert_eq!(g.config.joker_slots, slots_before);
}

#[test]
fn test_destroy_card() {
    let mut g = Game::default();
    g.deal();
    let card = g.available.cards()[0];
    let available_before = g.available.cards().len();
    g.destroy_card(card.id);
    assert_eq!(g.available.cards().len(), available_before - 1);
    assert!(g.available.cards().iter().all(|c| c.id != card.id));
}

#[test]
fn test_joker_state_mut_finds_by_instance_id() {
    use crate::joker::Jokers;
    use balatro_types::joker::TheJoker;

    let mut g = Game::default();
    let mut joker = Jokers::TheJoker(TheJoker::default());
    joker.set_instance_id(7);
    g.jokers.push(joker);

    let state = g.joker_state_mut(7).expect("joker with id 7 exists");
    state.counter = 3.5;

    assert_eq!(g.jokers[0].state().counter, 3.5);
}

#[test]
fn test_joker_state_mut_missing_id_returns_none() {
    use crate::joker::Jokers;
    use balatro_types::joker::TheJoker;

    let mut g = Game::default();
    let mut joker = Jokers::TheJoker(TheJoker::default());
    joker.set_instance_id(7);
    g.jokers.push(joker);

    assert!(g.joker_state_mut(999).is_none());
}

#[cfg(feature = "serde")]
#[test]
fn test_to_from_json_roundtrip() {
    let mut g = Game::default();
    g.start();
    let json = g.to_json().expect("serialize");
    let g2 = Game::from_json(&json).expect("deserialize");
    assert_eq!(g2.stage, g.stage);
    assert_eq!(g2.ante_current, g.ante_current);
    assert_eq!(g2.plays, g.plays);
    assert_eq!(g2.discards_remaining, g.discards_remaining);
    assert_eq!(g2.money, g.money);
    assert_eq!(g2.available.cards().len(), g.available.cards().len());
    assert_eq!(g2.deck.len(), g.deck.len());
}

#[cfg(feature = "serde")]
#[test]
fn test_from_json_restores_effect_registry() {
    use crate::joker::Jokers;
    use balatro_types::joker::HangingChad;
    let mut g = Game::default();
    g.start();
    let joker = Jokers::HangingChad(HangingChad::default());
    g.jokers.push(joker);
    let jokers = g.jokers.clone();
    g.effect_registry.register_jokers(jokers, &g.clone());
    assert!(!g.effect_registry.trigger_count_played.is_empty());

    let json = g.to_json().expect("serialize");
    let g2 = Game::from_json(&json).expect("deserialize");
    assert_eq!(g2.jokers.len(), 1);
    assert!(!g2.effect_registry.trigger_count_played.is_empty());
}

#[cfg(feature = "serde")]
#[test]
fn test_from_json_backfills_missing_current_boss() {
    let mut g = Game::default();
    g.start();
    assert!(g.current_boss.is_some());

    // Simulate a save written before `current_boss` existed by stripping
    // the field out of the serialized JSON.
    let mut value: serde_json::Value =
        serde_json::from_str(&g.to_json().expect("serialize")).expect("parse");
    value
        .as_object_mut()
        .expect("game serializes to an object")
        .remove("current_boss");
    let json = serde_json::to_string(&value).expect("reserialize");

    let g2 = Game::from_json(&json).expect("deserialize");
    assert!(g2.current_boss.is_some());
}

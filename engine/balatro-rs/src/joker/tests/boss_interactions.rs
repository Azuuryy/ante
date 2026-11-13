use super::*;

#[test]
fn test_reserved_parking_does_not_trigger_on_debuffed_held_face_card() {
    let mut g = Game {
        current_boss: Some(BossBlind::Plant),
        blind: Some(Blind::Boss),
        jokers: vec![Jokers::ReservedParking(ReservedParking::default())],
        ..Default::default()
    };
    let money_before = g.money;
    let king = Card::new(Value::King, Suit::Heart);
    g.available.extend(vec![king]); // held, not played

    let ace = Card::new(Value::Ace, Suit::Diamond);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    g.calc_score(hand);
    assert_eq!(g.money, money_before);
}

#[test]
fn test_baron_does_not_count_debuffed_held_king() {
    let mut g = Game {
        current_boss: Some(BossBlind::Plant),
        blind: Some(Blind::Boss),
        jokers: vec![Jokers::Baron(Baron::default())],
        ..Default::default()
    };
    let king = Card::new(Value::King, Suit::Heart);
    g.available.extend(vec![king]); // held, not played

    let ace = Card::new(Value::Ace, Suit::Diamond);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    // Baron never sees the held King (debuffed) -> mult stays 1.
    // (5 + 11) * 1 = 16, not * 1.5 = 24.
    assert_eq!(g.calc_score(hand), 16);
}

#[test]
fn test_matador_pays_out_when_boss_ability_triggers() {
    // Club is Matador-Yes (docs/boss-blinds.md §2): a played Club card
    // gets debuffed, which sets boss_triggered_this_hand.
    let mut g = Game {
        current_boss: Some(BossBlind::Club),
        blind: Some(Blind::Boss),
        jokers: vec![Jokers::Matador(Matador::default())],
        ..Default::default()
    };
    let money_before = g.money;
    let king = Card::new(Value::King, Suit::Club);
    let hand = SelectHand::new(vec![king]).best_hand().unwrap();
    g.calc_score(hand);
    assert_eq!(g.money, money_before + 8);
}

#[test]
fn test_matador_no_payout_without_boss_trigger() {
    let mut g = Game {
        current_boss: Some(BossBlind::Club),
        blind: Some(Blind::Boss),
        jokers: vec![Jokers::Matador(Matador::default())],
        ..Default::default()
    };
    let money_before = g.money;
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    g.calc_score(hand);
    assert_eq!(g.money, money_before);
}

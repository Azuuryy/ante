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

fn boss_game(boss: BossBlind) -> Game {
    Game {
        current_boss: Some(boss),
        blind: Some(Blind::Boss),
        jokers: vec![Jokers::Matador(Matador::default())],
        ..Default::default()
    }
}

fn cards(values: &[Value]) -> SelectHand {
    SelectHand::new(
        values
            .iter()
            .map(|&value| Card::new(value, Suit::Heart))
            .collect(),
    )
}

#[test]
fn the_psychic_refuses_fewer_than_five_cards_and_pays_matador() {
    let mut g = boss_game(BossBlind::Psychic);
    let money = g.money;
    let one = cards(&[Value::Three]).best_hand().unwrap();
    assert_eq!(g.calc_score(one), 0, "Not Allowed! scores nothing");
    assert!(g.boss_triggered_this_hand);
    assert_eq!(g.money, money + 8);

    let five = SelectHand::new(vec![
        Card::new(Value::Two, Suit::Heart),
        Card::new(Value::Five, Suit::Club),
        Card::new(Value::Seven, Suit::Spade),
        Card::new(Value::Nine, Suit::Diamond),
        Card::new(Value::Jack, Suit::Heart),
    ])
    .best_hand()
    .unwrap();
    assert!(g.calc_score(five) > 0);
    assert!(!g.boss_triggered_this_hand);
    assert_eq!(g.money, money + 8, "an allowed hand pays nothing");
}

#[test]
fn the_eye_refuses_a_hand_type_already_played_this_round() {
    let mut g = boss_game(BossBlind::Eye);
    let pair = || cards(&[Value::King, Value::King]).best_hand().unwrap();
    assert!(g.calc_score(pair()) > 0);
    assert_eq!(g.calc_score(pair()), 0);
    assert!(g.boss_triggered_this_hand);
    let high = cards(&[Value::Ace]).best_hand().unwrap();
    assert!(g.calc_score(high) > 0);
}

#[test]
fn the_mouth_only_allows_the_first_hand_type_of_the_round() {
    let mut g = boss_game(BossBlind::Mouth);
    let high = || cards(&[Value::Ace]).best_hand().unwrap();
    assert!(g.calc_score(high()) > 0);
    let pair = cards(&[Value::King, Value::King]).best_hand().unwrap();
    assert_eq!(g.calc_score(pair), 0);
    assert!(
        g.calc_score(high()) > 0,
        "the first hand type stays allowed"
    );
    g.clear_blind();
    assert_eq!(g.mouth_hand, None);
}

#[test]
fn a_disabled_boss_allows_every_hand() {
    let mut g = boss_game(BossBlind::Psychic);
    g.boss_disabled_by_luchador = true;
    let one = cards(&[Value::Three]).best_hand().unwrap();
    assert!(g.calc_score(one) > 0);
}

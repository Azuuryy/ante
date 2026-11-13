use super::*;

#[test]
fn test_boss_triggered_this_hand_set_on_debuff() {
    let mut g = Game {
        current_boss: Some(BossBlind::Club),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    let king = Card::new(Value::King, Suit::Club);
    let hand = SelectHand::new(vec![king]).best_hand().unwrap();
    g.calc_score(hand);
    assert!(g.boss_triggered_this_hand);
}

#[test]
fn test_boss_triggered_this_hand_false_without_debuff() {
    let mut g = Game {
        current_boss: Some(BossBlind::Club),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    g.calc_score(hand);
    assert!(!g.boss_triggered_this_hand);
}

#[test]
fn test_boss_triggered_this_hand_resets_each_hand() {
    let mut g = Game {
        current_boss: Some(BossBlind::Club),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    let king = Card::new(Value::King, Suit::Club);
    let hand = SelectHand::new(vec![king]).best_hand().unwrap();
    g.calc_score(hand);
    assert!(g.boss_triggered_this_hand);

    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand2 = SelectHand::new(vec![ace]).best_hand().unwrap();
    g.calc_score(hand2);
    assert!(!g.boss_triggered_this_hand);
}

#[test]
fn test_boss_triggered_this_hand_false_for_debuffed_held_card_without_enhancement() {
    // A bare (no enhancement) debuffed held card was never going to
    // contribute anything in the held-card loop - the debuff didn't
    // actually block anything, so it shouldn't pay Matador.
    let mut g = Game {
        current_boss: Some(BossBlind::Club),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    let held_king = Card::new(Value::King, Suit::Club);
    g.available.extend(vec![held_king]);
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    g.calc_score(hand);
    assert!(!g.boss_triggered_this_hand);
}

#[test]
fn test_boss_triggered_this_hand_true_for_debuffed_held_gold_card() {
    let mut g = Game {
        current_boss: Some(BossBlind::Club),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    let mut held_king = Card::new(Value::King, Suit::Club);
    held_king.enhancement = Some(Enhancement::Gold);
    g.available.extend(vec![held_king]);
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    g.calc_score(hand);
    assert!(g.boss_triggered_this_hand);
}

// --- Group B: required-score multiplier (Wall, VioletVessel, Needle) ---

#[test]
fn test_wall_boss_quadruples_required_score() {
    let g = Game {
        current_boss: Some(BossBlind::Wall),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    assert_eq!(g.required_score(), 300 * 4);
}

#[test]
fn test_violet_vessel_boss_sextuples_required_score() {
    let g = Game {
        current_boss: Some(BossBlind::VioletVessel),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    assert_eq!(g.required_score(), 300 * 6);
}

#[test]
fn test_needle_boss_reduces_required_score() {
    let g = Game {
        current_boss: Some(BossBlind::Needle),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    assert_eq!(g.required_score(), 300);
}

#[test]
fn test_required_score_default_boss_multiplier_for_other_bosses() {
    let g = Game {
        current_boss: Some(BossBlind::Club),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    assert_eq!(g.required_score(), 300 * 2);
}

#[test]
fn test_required_score_returns_to_default_when_boss_disabled() {
    let g = Game {
        current_boss: Some(BossBlind::Wall),
        blind: Some(Blind::Boss),
        boss_disabled_by_luchador: true,
        ..Default::default()
    };
    assert_eq!(g.required_score(), 300 * 2);
}

#[test]
fn test_boss_required_score_previews_before_boss_blind_selected() {
    let g = Game {
        current_boss: Some(BossBlind::Wall),
        blind: Some(Blind::Big),
        stage: Stage::PreBlind(),
        ..Default::default()
    };
    // required_score() reflects the *current* blind (Big), not Boss -
    // boss_required_score() previews the upcoming Boss's number instead.
    assert_eq!(g.required_score(), (300f32 * 1.5) as usize);
    assert_eq!(g.boss_required_score(), 300 * 4);
}

// --- Group C: hand-level chip/mult halving (Flint) ---

#[test]
fn test_flint_boss_halves_pair_base_chips_and_mult() {
    let mut g = Game {
        current_boss: Some(BossBlind::Flint),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    let king1 = Card::new(Value::King, Suit::Heart);
    let king2 = Card::new(Value::King, Suit::Diamond);
    let hand = SelectHand::new(vec![king1, king2]).best_hand().unwrap();
    // OnePair level 1: (10, 2) halved to (5, 1). Cards unaffected: +20 chips.
    // (5 + 20) * 1 = 25.
    assert_eq!(g.calc_score(hand), 25);
}

#[test]
fn test_flint_boss_rounds_halves_to_nearest_with_mult_at_least_one() {
    let mut g = Game {
        current_boss: Some(BossBlind::Flint),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    // HighCard level 1: (5, 1) -> chips floor(2.5 + 0.5) = 3, mult max(1, 1) = 1
    // like `Blind:modify_hand`. (3 + 11) * 1 = 14.
    assert_eq!(g.calc_score(hand), 14);
}

#[test]
fn test_flint_boss_sets_boss_triggered_this_hand() {
    let mut g = Game {
        current_boss: Some(BossBlind::Flint),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    g.calc_score(hand);
    assert!(g.boss_triggered_this_hand);
}

// --- Group D: delevel by one (Arm) ---

#[test]
fn test_arm_boss_decrements_level_by_one_not_to_one() {
    let mut g = Game {
        current_boss: Some(BossBlind::Arm),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    g.planetarium.level_up(HandRank::OnePair);
    g.planetarium.level_up(HandRank::OnePair);
    // level 3: (40, 4) -> Arm decrements once to level 2: (25, 3).
    let king1 = Card::new(Value::King, Suit::Heart);
    let king2 = Card::new(Value::King, Suit::Diamond);
    let hand = SelectHand::new(vec![king1, king2]).best_hand().unwrap();
    // (25 + 20) * 3 = 135, NOT level-1 (10 + 20) * 2 = 60.
    assert_eq!(g.calc_score(hand), 135);
}

#[test]
fn test_arm_boss_decrement_is_permanent() {
    let mut g = Game {
        current_boss: Some(BossBlind::Arm),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    g.planetarium.level_up(HandRank::OnePair);
    g.planetarium.level_up(HandRank::OnePair);
    let king1 = Card::new(Value::King, Suit::Heart);
    let king2 = Card::new(Value::King, Suit::Diamond);
    let hand = SelectHand::new(vec![king1, king2]).best_hand().unwrap();
    g.calc_score(hand);
    assert_eq!(g.planetarium.level(HandRank::OnePair).level, 2);
}

#[test]
fn test_arm_boss_cannot_decrement_below_level_one() {
    let mut g = Game {
        current_boss: Some(BossBlind::Arm),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    let king1 = Card::new(Value::King, Suit::Heart);
    let king2 = Card::new(Value::King, Suit::Diamond);
    let hand = SelectHand::new(vec![king1, king2]).best_hand().unwrap();
    // Already level 1: (10 + 20) * 2 = 60, unaffected.
    assert_eq!(g.calc_score(hand), 60);
    assert_eq!(g.planetarium.level(HandRank::OnePair).level, 1);
}

#[test]
fn test_arm_boss_decrements_again_each_subsequent_play_until_level_one() {
    let mut g = Game {
        current_boss: Some(BossBlind::Arm),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    g.planetarium.level_up(HandRank::OnePair);
    g.planetarium.level_up(HandRank::OnePair);
    let hand = || {
        let king1 = Card::new(Value::King, Suit::Heart);
        let king2 = Card::new(Value::King, Suit::Diamond);
        SelectHand::new(vec![king1, king2]).best_hand().unwrap()
    };

    g.calc_score(hand()); // level 3 -> 2
    assert!(g.boss_triggered_this_hand);
    assert_eq!(g.planetarium.level(HandRank::OnePair).level, 2);

    g.calc_score(hand()); // level 2 -> 1
    assert!(g.boss_triggered_this_hand);
    assert_eq!(g.planetarium.level(HandRank::OnePair).level, 1);

    g.calc_score(hand()); // already level 1, no further decrement
    assert!(!g.boss_triggered_this_hand);
    assert_eq!(g.planetarium.level(HandRank::OnePair).level, 1);
}

#[test]
fn test_arm_boss_triggers_matador_only_when_actually_deleveled() {
    let mut g = Game {
        current_boss: Some(BossBlind::Arm),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    let king1 = Card::new(Value::King, Suit::Heart);
    let king2 = Card::new(Value::King, Suit::Diamond);
    let hand = SelectHand::new(vec![king1, king2]).best_hand().unwrap();
    g.calc_score(hand);
    assert!(!g.boss_triggered_this_hand);
}

#[test]
fn test_arm_boss_still_increments_real_play_count() {
    let mut g = Game {
        current_boss: Some(BossBlind::Arm),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    g.planetarium.level_up(HandRank::OnePair);
    let king1 = Card::new(Value::King, Suit::Heart);
    let king2 = Card::new(Value::King, Suit::Diamond);
    let hand = SelectHand::new(vec![king1, king2]).best_hand().unwrap();
    g.calc_score(hand);
    assert_eq!(g.planetarium.level(HandRank::OnePair).plays, 1);
}

// --- Group E, part 1: Water, Manacle, Serpent ---

#[test]
fn test_water_boss_blocks_discards() {
    let g = Game {
        current_boss: Some(BossBlind::Water),
        blind: Some(Blind::Boss),
        discards_remaining: 4,
        ..Default::default()
    };
    assert_eq!(g.discards(), 0);
    assert_eq!(g.discards_remaining, 4);
}

#[test]
fn test_water_boss_discards_restored_when_disabled_mid_round() {
    let g = Game {
        current_boss: Some(BossBlind::Water),
        blind: Some(Blind::Boss),
        discards_remaining: 4,
        boss_disabled_by_luchador: true,
        ..Default::default()
    };
    assert_eq!(g.discards(), 4);
}

#[test]
fn test_water_boss_no_effect_outside_boss_round() {
    let g = Game {
        current_boss: Some(BossBlind::Water),
        blind: Some(Blind::Small),
        discards_remaining: 4,
        ..Default::default()
    };
    assert_eq!(g.discards(), 4);
}

#[test]
fn test_discard_selected_rejected_under_water() {
    let mut g = Game {
        current_boss: Some(BossBlind::Water),
        blind: Some(Blind::Big),
        stage: Stage::PreBlind(),
        ..Default::default()
    };
    g.select_blind(Blind::Boss).expect("can select boss blind");
    let card = g.available.cards()[0];
    g.select_card(card).expect("can select card");
    let before = g.discards_remaining;
    let result = g.discard_selected();
    assert!(matches!(result, Err(GameError::NoRemainingDiscards)));
    assert_eq!(g.discards_remaining, before);
}

#[test]
fn test_manacle_boss_reduces_hand_size() {
    let g = Game {
        current_boss: Some(BossBlind::Manacle),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    assert_eq!(g.hand_size(), 7);
}

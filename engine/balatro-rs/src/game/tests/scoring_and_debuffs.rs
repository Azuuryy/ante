use super::*;

#[test]
fn test_calc_score_traced_hand_and_played_card_steps() {
    let mut g = Game::default();
    let ace = Card::new(Value::Ace, Suit::Heart);
    let king = Card::new(Value::King, Suit::Diamond);
    let jack = Card::new(Value::Jack, Suit::Club);

    // Same fixture as test_calc_score's first case, best hand is the ace alone.
    let hand = SelectHand::new(vec![ace, king, jack]).best_hand().unwrap();
    let (score, trace) = g.calc_score_traced(hand);
    assert_eq!(score, 16);

    assert_eq!(trace.0.len(), 2);

    assert_eq!(
        trace.0[0].source,
        ScoreSource::HandLevel(HandRank::HighCard)
    );
    assert_eq!(trace.0[0].chips_before, 0);
    assert_eq!(trace.0[0].chips_after, 5);
    assert_eq!(trace.0[0].mult_before, 0.0);
    assert_eq!(trace.0[0].mult_after, 1.0);
    assert!(!trace.0[0].retrigger);

    assert_eq!(trace.0[1].source, ScoreSource::PlayedCard(ace));
    assert_eq!(trace.0[1].chips_before, 5);
    assert_eq!(trace.0[1].chips_after, 16);
    assert_eq!(trace.0[1].mult_before, 1.0);
    assert_eq!(trace.0[1].mult_after, 1.0);
    assert!(!trace.0[1].retrigger);
}

#[test]
fn test_calc_score_traced_held_card_steps() {
    let mut g = Game::default();
    let ace = Card::new(Value::Ace, Suit::Heart);
    let mut steel_king = Card::new(Value::King, Suit::Diamond);
    steel_king.enhancement = Some(Enhancement::Steel);

    // Held, not played — g.available.extend() leaves cards unselected.
    g.available.extend(vec![steel_king]);

    // mult=2 baseline so Steel's `mult += mult/2` has a nonzero result.
    let hand = SelectHand::new(vec![ace, ace]).best_hand().unwrap();
    let (score, trace) = g.calc_score_traced(hand);

    let held_steps: Vec<_> = trace
        .0
        .iter()
        .filter(|s| matches!(s.source, ScoreSource::HeldCard(_)))
        .collect();
    assert_eq!(held_steps.len(), 1);
    assert_eq!(held_steps[0].source, ScoreSource::HeldCard(steel_king));
    assert_eq!(held_steps[0].mult_before, 2.0);
    assert_eq!(held_steps[0].mult_after, 3.0);
    assert_eq!(held_steps[0].chips_before, held_steps[0].chips_after);

    // Pair (level 1): 10 chips, 2 mult. Played (2 aces): +11+11 chips.
    // Held Steel king: mult 2 -> 3. (10 + 22) * 3 = 96.
    assert_eq!(score, 96);
}

#[test]
fn test_handle_score() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Blind(Blind::Small);
    g.blind = Some(Blind::Small);

    // Not enough to pass
    let required = g.required_score();
    let score = required - 1;

    let passed = g.handle_score(score).unwrap();
    assert!(!passed);
    assert_eq!(g.score, score);

    // Enough to pass now
    let passed = g.handle_score(1).unwrap();
    assert!(passed);
    assert_eq!(g.score, required);
    assert_eq!(g.stage, Stage::PostBlind());
}

#[test]
fn test_clear_blind() {
    let mut g = Game::default();
    g.start();
    g.deal();
    g.clear_blind();
    // all cards return to deck, available is empty
    assert_eq!(g.deck.len(), 52);
    assert_eq!(g.available.cards().len(), 0);
}

#[test]
fn test_active_boss_during_boss_blind() {
    let g = Game {
        current_boss: Some(BossBlind::Club),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    assert_eq!(g.active_boss(), Some(BossBlind::Club));
}

#[test]
fn test_active_boss_none_outside_boss_round() {
    let g = Game {
        current_boss: Some(BossBlind::Club),
        blind: Some(Blind::Small),
        ..Default::default()
    };
    assert_eq!(g.active_boss(), None);
}

#[test]
fn test_active_boss_none_when_chicot_held() {
    use crate::joker::*;
    let g = Game {
        current_boss: Some(BossBlind::Club),
        blind: Some(Blind::Boss),
        jokers: vec![Jokers::Chicot(Chicot::default())],
        ..Default::default()
    };
    assert_eq!(g.active_boss(), None);
}

#[test]
fn test_active_boss_none_when_luchador_disabled() {
    let g = Game {
        current_boss: Some(BossBlind::Club),
        blind: Some(Blind::Boss),
        boss_disabled_by_luchador: true,
        ..Default::default()
    };
    assert_eq!(g.active_boss(), None);
}

#[test]
fn test_boss_disabled_by_luchador_resets_on_clear_blind() {
    let mut g = Game {
        boss_disabled_by_luchador: true,
        ..Default::default()
    };
    g.clear_blind();
    assert!(!g.boss_disabled_by_luchador);
}

#[test]
fn test_club_boss_debuffs_club_cards() {
    // King alone: (5 + 10) * 1 = 15 baseline; debuffed, contributes 0 -> 5.
    let mut g = Game {
        current_boss: Some(BossBlind::Club),
        blind: Some(Blind::Boss),
        ..Default::default()
    };

    // doesn't debuff heart
    let king_h = Card::new(Value::King, Suit::Heart);
    let hand = SelectHand::new(vec![king_h]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 15);

    // debuffs club
    let king_c = Card::new(Value::King, Suit::Club);
    let hand = SelectHand::new(vec![king_c]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 5);
}

#[test]
fn test_goad_boss_debuffs_spade_cards() {
    let mut g = Game {
        current_boss: Some(BossBlind::Goad),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    let king = Card::new(Value::King, Suit::Spade);
    let hand = SelectHand::new(vec![king]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 5);
}

#[test]
fn test_head_boss_debuffs_heart_cards() {
    let mut g = Game {
        current_boss: Some(BossBlind::Head),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    let king = Card::new(Value::King, Suit::Heart);
    let hand = SelectHand::new(vec![king]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 5);
}

#[test]
fn test_window_boss_debuffs_diamond_cards() {
    let mut g = Game {
        current_boss: Some(BossBlind::Window),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    let king = Card::new(Value::King, Suit::Diamond);
    let hand = SelectHand::new(vec![king]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 5);
}

#[test]
fn test_plant_boss_debuffs_face_cards() {
    let mut g = Game {
        current_boss: Some(BossBlind::Plant),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    let king = Card::new(Value::King, Suit::Heart);
    let king_hand = SelectHand::new(vec![king]).best_hand().unwrap();
    assert_eq!(g.calc_score(king_hand), 5);

    // Non-face card in the same round, unaffected by Plant.
    // (5 + 10) * 1 = 15.
    let ten = Card::new(Value::Ten, Suit::Heart);
    let ten_hand = SelectHand::new(vec![ten]).best_hand().unwrap();
    assert_eq!(g.calc_score(ten_hand), 15);
}

#[test]
fn test_wild_card_debuffed_by_suit_boss_despite_different_suit() {
    let mut g = Game {
        current_boss: Some(BossBlind::Window),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    // Wild matches every suit, so it's debuffed by the
    // Diamond-debuffing Window boss despite its literal suit (Heart).
    let mut wild_ace = Card::new(Value::Ace, Suit::Heart);
    wild_ace.enhancement = Some(Enhancement::Wild);
    let wild_hand = SelectHand::new(vec![wild_ace]).best_hand().unwrap();
    assert_eq!(g.calc_score(wild_hand), 5);

    // Control: a genuinely Heart (non-Wild) card isn't debuffed by Window.
    // (5 + 11) * 1 = 16.
    let plain_ace = Card::new(Value::Ace, Suit::Heart);
    let plain_hand = SelectHand::new(vec![plain_ace]).best_hand().unwrap();
    assert_eq!(g.calc_score(plain_hand), 16);
}

#[test]
fn test_stone_kicker_is_not_debuffed_by_a_suit_boss() {
    // A Stone card has no suit (`Card:is_suit` returns false), so The Club
    // does not debuff a Club-based Stone card: its +50 chips still score.
    // (5 + 11 + 50) * 1 = 66.
    let mut g = Game {
        current_boss: Some(BossBlind::Club),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    let ace = Card::new(Value::Ace, Suit::Heart);
    let mut stone = Card::new(Value::Two, Suit::Club);
    stone.enhancement = Some(Enhancement::Stone);
    let hand = SelectHand::new(vec![ace, stone]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 66);
}

#[test]
fn test_debuff_only_applies_during_boss_round() {
    // Same Club King as test_club_boss_debuffs_club_cards, but blind is
    // Small, not Boss - active_boss() returns None (Step 2's
    // round-scoping gate), so the debuff never applies.
    let mut g = Game {
        current_boss: Some(BossBlind::Club),
        blind: Some(Blind::Small),
        ..Default::default()
    };
    let king = Card::new(Value::King, Suit::Club);
    let hand = SelectHand::new(vec![king]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 15);
}

#[test]
fn test_debuffed_card_still_counts_for_hand_type() {
    // Two Club Kings still form a Pair even though both are debuffed -
    // hand-type detection happens upstream of calc_score_inner. Pair
    // level 1: chips=10, mult=2; neither King contributes chips ->
    // 10 * 2 = 20.
    let mut g = Game {
        current_boss: Some(BossBlind::Club),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    let king1 = Card::new(Value::King, Suit::Club);
    let king2 = Card::new(Value::King, Suit::Club);
    let hand = SelectHand::new(vec![king1, king2]).best_hand().unwrap();
    assert_eq!(hand.rank, HandRank::OnePair);
    assert_eq!(g.calc_score(hand), 20);
}

#[test]
fn test_debuffed_card_loses_seal_money() {
    let mut g = Game {
        current_boss: Some(BossBlind::Club),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    let money_before = g.money;
    let mut king = Card::new(Value::King, Suit::Club);
    king.seal = Some(Seal::Gold);
    let hand = SelectHand::new(vec![king]).best_hand().unwrap();
    g.calc_score(hand);
    assert_eq!(g.money, money_before);
}

#[test]
fn test_debuffed_held_card_loses_enhancement() {
    let mut g = Game {
        current_boss: Some(BossBlind::Club),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    let money_before = g.money;
    let ace = Card::new(Value::Ace, Suit::Heart);
    let mut gold_king = Card::new(Value::King, Suit::Club);
    gold_king.enhancement = Some(Enhancement::Gold);
    g.available.extend(vec![gold_king]);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    g.calc_score(hand);
    assert_eq!(g.money, money_before);
}

#[test]
fn test_debuffed_card_skips_retrigger() {
    // Red Seal normally doubles trigger_count_played; under the Club
    // boss, the debuffed card's contribution (and its retrigger) are
    // skipped entirely - score matches the bare base level, not a
    // doubled-then-zeroed contribution.
    let mut g = Game {
        current_boss: Some(BossBlind::Club),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    let mut king = Card::new(Value::King, Suit::Club);
    king.seal = Some(Seal::Red);
    let hand = SelectHand::new(vec![king]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 5);
}

#[test]
fn test_non_debuffed_filters_out_debuffed_cards() {
    let g = Game {
        current_boss: Some(BossBlind::Club),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    let club_king = Card::new(Value::King, Suit::Club);
    let heart_ace = Card::new(Value::Ace, Suit::Heart);
    let cards = [club_king, heart_ace];
    assert_eq!(g.non_debuffed(cards.iter()), vec![heart_ace]);
}

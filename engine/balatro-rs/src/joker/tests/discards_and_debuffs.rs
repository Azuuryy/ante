use super::*;

#[test]
fn test_mail_in_rebate_pays_per_matching_rank_discarded() {
    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.money += 1000;
    g.stage = Stage::Shop();
    let mut j = Jokers::MailInRebate(MailInRebate::default());
    j.set_instance_id(1);
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    g.joker_state_mut(1).unwrap().selector = Some(SelectorValue::Value(Value::King));

    let k1 = Card::new(Value::King, Suit::Heart);
    let k2 = Card::new(Value::King, Suit::Spade);
    let three = Card::new(Value::Three, Suit::Club);
    g.available.extend(vec![k1, k2, three]);
    g.select_card(k1).unwrap();
    g.select_card(k2).unwrap();
    g.select_card(three).unwrap();
    let money_before = g.money;
    g.discard_selected().unwrap();
    // 2 kings discarded in this action: +$5 each
    assert_eq!(g.money, money_before + 10);

    let k3 = Card::new(Value::King, Suit::Diamond);
    g.available.extend(vec![k3]);
    g.select_card(k3).unwrap();
    let money_before = g.money;
    g.discard_selected().unwrap();
    // immediate per-event payout, not accumulated from prior discards
    assert_eq!(g.money, money_before + 5);
}

#[test]
fn test_discard_selectors_reroll_on_clear_blind() {
    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.money += 1000;
    g.stage = Stage::Shop();
    let mut castle = Jokers::Castle(Castle::default());
    castle.set_instance_id(1);
    g.shop.jokers.push(castle.clone());
    g.buy_joker(castle).unwrap();
    let mut mail = Jokers::MailInRebate(MailInRebate::default());
    mail.set_instance_id(2);
    g.shop.jokers.push(mail.clone());
    g.buy_joker(mail).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    assert_eq!(g.joker_state_mut(1).unwrap().selector, None);
    assert_eq!(g.joker_state_mut(2).unwrap().selector, None);

    g.clear_blind();

    assert!(g.joker_state_mut(1).unwrap().selector.is_some());
    assert!(g.joker_state_mut(2).unwrap().selector.is_some());
}

#[test]
fn test_roll_discard_selector_sets_castle_and_mail_in_rebate() {
    // `roll_discard_selector` is called directly inside both real mint
    // chokepoints (`JokerGenerator::gen_joker`, `rng::seed_joker_with_id`)
    // - tested here directly rather than via `gen_joker`'s own random
    // rarity/pool selection. An earlier version of this test sampled
    // `gen_joker` in a loop hoping to draw both jokers within N tries,
    // which ties the test's pass/fail to incidental RNG-sequence
    // details - a future change to the joker roster or rarity weights
    // could shift whether a fixed seed lands on both within a bounded
    // number of draws, failing this test for no real regression.
    use rand::SeedableRng;
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(7);

    let mut castle = Jokers::Castle(Castle::default());
    roll_discard_selector(&mut rng, &mut castle);
    assert!(
        castle.state().selector.is_some(),
        "Castle minted with no selector"
    );

    let mut mail = Jokers::MailInRebate(MailInRebate::default());
    roll_discard_selector(&mut rng, &mut mail);
    assert!(
        mail.state().selector.is_some(),
        "MailInRebate minted with no selector"
    );
}

// --- Group A boss debuff x joker interactions ---
// Each test plays/holds/discards a card that would normally trigger the
// joker, but the card is debuffed by the relevant Group A boss (via
// `non_debuffed`) - the joker's bonus must not apply.

#[test]
fn test_greedy_joker_does_not_count_debuffed_diamond() {
    let mut g = Game {
        current_boss: Some(BossBlind::Window),
        blind: Some(Blind::Boss),
        jokers: vec![Jokers::GreedyJoker(GreedyJoker::default())],
        ..Default::default()
    };
    let ace = Card::new(Value::Ace, Suit::Diamond);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    // (5) * 1 = 5 - debuffed card contributes nothing and isn't counted.
    assert_eq!(g.calc_score(hand), 5);
}

#[test]
fn test_lusty_joker_does_not_count_debuffed_heart() {
    let mut g = Game {
        current_boss: Some(BossBlind::Head),
        blind: Some(Blind::Boss),
        jokers: vec![Jokers::LustyJoker(LustyJoker::default())],
        ..Default::default()
    };
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 5);
}

#[test]
fn test_wrathful_joker_does_not_count_debuffed_spade() {
    let mut g = Game {
        current_boss: Some(BossBlind::Goad),
        blind: Some(Blind::Boss),
        jokers: vec![Jokers::WrathfulJoker(WrathfulJoker::default())],
        ..Default::default()
    };
    let ace = Card::new(Value::Ace, Suit::Spade);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 5);
}

#[test]
fn test_gluttonous_joker_does_not_count_debuffed_club() {
    let mut g = Game {
        current_boss: Some(BossBlind::Club),
        blind: Some(Blind::Boss),
        jokers: vec![Jokers::GluttonousJoker(GluttonousJoker::default())],
        ..Default::default()
    };
    let ace = Card::new(Value::Ace, Suit::Club);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 5);
}

#[test]
fn test_rough_gem_does_not_pay_for_debuffed_diamond() {
    let mut g = Game {
        current_boss: Some(BossBlind::Window),
        blind: Some(Blind::Boss),
        jokers: vec![Jokers::RoughGem(RoughGem::default())],
        ..Default::default()
    };
    let money_before = g.money;
    let ace = Card::new(Value::Ace, Suit::Diamond);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    g.calc_score(hand);
    assert_eq!(g.money, money_before);
}

#[test]
fn test_bloodstone_does_not_trigger_on_debuffed_heart() {
    let mut g = Game {
        current_boss: Some(BossBlind::Head),
        blind: Some(Blind::Boss),
        jokers: vec![Jokers::Bloodstone(Bloodstone::default())],
        ..Default::default()
    };
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    // debuffed card is filtered before the probabilistic roll ever
    // happens, so this is deterministic despite Bloodstone's own RNG.
    assert_eq!(g.calc_score(hand), 5);
}

#[test]
fn test_arrowhead_does_not_add_chips_for_debuffed_spade() {
    let mut g = Game {
        current_boss: Some(BossBlind::Goad),
        blind: Some(Blind::Boss),
        jokers: vec![Jokers::Arrowhead(Arrowhead::default())],
        ..Default::default()
    };
    let ace = Card::new(Value::Ace, Suit::Spade);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 5);
}

#[test]
fn test_onyx_agate_does_not_add_mult_for_debuffed_club() {
    let mut g = Game {
        current_boss: Some(BossBlind::Club),
        blind: Some(Blind::Boss),
        jokers: vec![Jokers::OnyxAgate(OnyxAgate::default())],
        ..Default::default()
    };
    let ace = Card::new(Value::Ace, Suit::Club);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 5);
}

#[test]
fn test_castle_does_not_count_debuffed_discarded_suit() {
    let mut j = Jokers::Castle(Castle::default());
    j.set_instance_id(1);
    let mut g = Game {
        current_boss: Some(BossBlind::Club),
        blind: Some(Blind::Boss),
        jokers: vec![j],
        ..Default::default()
    };
    g.joker_state_mut(1).unwrap().selector = Some(SelectorValue::Suit(Suit::Club));

    let club_card = Card::new(Value::Four, Suit::Club);
    g.available.extend(vec![club_card]);
    g.select_card(club_card).unwrap();
    g.discard_selected().unwrap();

    let ace_hand = SelectHand::new(vec![Card::new(Value::Ace, Suit::Heart)])
        .best_hand()
        .unwrap();
    // debuffed discarded Club doesn't count -> Castle contributes 0
    // chips. (5 + 11) * 1 = 16, not 16 + 3 = 19.
    assert_eq!(g.calc_score(ace_hand), 16);
}

#[test]
fn test_hit_the_road_does_not_count_debuffed_discarded_jack() {
    let mut g = Game {
        current_boss: Some(BossBlind::Club),
        blind: Some(Blind::Boss),
        jokers: vec![Jokers::HitTheRoad(HitTheRoad::default())],
        ..Default::default()
    };
    let jack_club = Card::new(Value::Jack, Suit::Club);
    g.available.extend(vec![jack_club]);
    g.select_card(jack_club).unwrap();
    g.discard_selected().unwrap();

    let ace_hand = SelectHand::new(vec![Card::new(Value::Ace, Suit::Heart)])
        .best_hand()
        .unwrap();
    // debuffed discarded Jack doesn't count -> HitTheRoad contributes
    // 0 mult. (5 + 11) * 1 = 16, not (5 + 11) * 1.5 = 24.
    assert_eq!(g.calc_score(ace_hand), 16);
}

#[test]
fn test_smiley_face_does_not_count_debuffed_face_card() {
    let mut g = Game {
        current_boss: Some(BossBlind::Plant),
        blind: Some(Blind::Boss),
        jokers: vec![Jokers::SmileyFace(SmileyFace::default())],
        ..Default::default()
    };
    let king = Card::new(Value::King, Suit::Heart);
    let hand = SelectHand::new(vec![king]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 5);
}

#[test]
fn test_scary_face_does_not_add_chips_for_debuffed_face_card() {
    let mut g = Game {
        current_boss: Some(BossBlind::Plant),
        blind: Some(Blind::Boss),
        jokers: vec![Jokers::ScaryFace(ScaryFace::default())],
        ..Default::default()
    };
    let king = Card::new(Value::King, Suit::Heart);
    let hand = SelectHand::new(vec![king]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 5);
}

#[test]
fn test_business_card_does_not_trigger_on_debuffed_face_card() {
    let mut g = Game {
        current_boss: Some(BossBlind::Plant),
        blind: Some(Blind::Boss),
        jokers: vec![Jokers::BusinessCard(BusinessCard::default())],
        ..Default::default()
    };
    let money_before = g.money;
    let king = Card::new(Value::King, Suit::Heart);
    let hand = SelectHand::new(vec![king]).best_hand().unwrap();
    g.calc_score(hand);
    assert_eq!(g.money, money_before);
}

#[test]
fn test_photograph_does_not_double_mult_for_debuffed_face_card() {
    let mut g = Game {
        current_boss: Some(BossBlind::Plant),
        blind: Some(Blind::Boss),
        jokers: vec![Jokers::Photograph(Photograph::default())],
        ..Default::default()
    };
    let king = Card::new(Value::King, Suit::Heart);
    let hand = SelectHand::new(vec![king]).best_hand().unwrap();
    // mult stays at 1 (never doubled) -> 5 * 1 = 5, not 5 * 2 = 10.
    assert_eq!(g.calc_score(hand), 5);
}

#[test]
fn test_ride_the_bus_ignores_debuffed_face_card() {
    let mut g = Game {
        current_boss: Some(BossBlind::Plant),
        blind: Some(Blind::Boss),
        jokers: vec![Jokers::RideTheBus(RideTheBus::default())],
        ..Default::default()
    };
    let king = Card::new(Value::King, Suit::Heart);
    let hand = SelectHand::new(vec![king]).best_hand().unwrap();
    // debuffed King doesn't count as a face card -> streak 0 -> 1,
    // +1 mult. chips = 5 (debuffed), mult = 1 + 1 = 2 -> 5 * 2 = 10.
    // If the debuff were ignored here, the King would reset the streak
    // instead (mult stays 1, score 5).
    assert_eq!(g.calc_score(hand), 10);
}

#[test]
fn test_midas_mask_does_not_convert_debuffed_face_card() {
    let mut g = Game {
        current_boss: Some(BossBlind::Plant),
        blind: Some(Blind::Boss),
        jokers: vec![Jokers::MidasMask(MidasMask::default())],
        ..Default::default()
    };
    let king = Card::new(Value::King, Suit::Heart);
    g.available.extend(vec![king]);

    let hand = SelectHand::new(vec![king]);
    g.calc_score(hand.best_hand().unwrap());

    let scored = g.available.cards().into_iter().find(|c| c.id == king.id);
    assert_eq!(scored.unwrap().enhancement, None);
}

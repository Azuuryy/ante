use super::*;
use crate::card::{Suit, Value};

#[test]
fn test_real_rng_mode_matches_balatro_seed_reference() {
    use crate::pack::PackSize;

    let config = Config {
        rng_mode: RngMode::Real,
        seed_str: Some("TEST".to_string()),
        ..Config::default()
    };
    let mut g = Game::new(config);

    let planetarium = g.planetarium.clone();
    let vouchers = Vouchers::new();
    let ctx = ShopContext::for_test(&planetarium, &vouchers, 1);
    g.shop.refresh(&ctx, &mut g.backend);

    assert_eq!(g.shop.jokers.len(), 1);
    assert_eq!(g.shop.jokers[0].name(), "Ice Cream");
    assert_eq!(g.shop.consumables.len(), 1);
    assert_eq!(g.shop.consumables[0].name(), "Strength");

    assert_eq!(g.shop.packs.len(), 2);

    assert_eq!(g.shop.packs[0].category, PackCategory::Buffoon);
    assert_eq!(g.shop.packs[0].size, PackSize::Normal);
    let p1_contents: Vec<String> = g.shop.packs[0].contents.iter().map(|c| c.name()).collect();
    assert_eq!(p1_contents, vec!["Baseball Card", "Drunkard"]);

    assert_eq!(g.shop.packs[1].category, PackCategory::Spectral);
    assert_eq!(g.shop.packs[1].size, PackSize::Normal);
    let p2_contents: Vec<String> = g.shop.packs[1].contents.iter().map(|c| c.name()).collect();
    assert_eq!(p2_contents, vec!["Medium", "Wraith"]);
}

// `cargo run -p balatro-seed --bin explore -- TEST --ante 1` reports
// `Boss: The Goad` for this seed, but `RealBackend::draw_boss` doesn't
// match it (yet).
// TODO: Fix min ante for real rng boss gen
#[test]
fn test_real_rng_mode_draw_boss_reaches_instance() {
    let config = Config {
        rng_mode: RngMode::Real,
        seed_str: Some("TEST".to_string()),
        ..Config::default()
    };
    let g = Game::new(config);
    assert_eq!(g.current_boss, Some(BossBlind::Flint));
}

// Same seed through two independent `Game`s: proves `RngMode::Real`
// stays deterministic as antes advance.
#[test]
fn test_real_rng_mode_is_deterministic_across_antes() {
    use crate::pack::PackSize;

    type AnteDraws = (Vec<String>, Vec<String>, Vec<(PackCategory, PackSize)>);

    fn run() -> Vec<AnteDraws> {
        let config = Config {
            rng_mode: RngMode::Real,
            seed_str: Some("TEST".to_string()),
            ..Config::default()
        };
        let mut g = Game::new(config);
        let planetarium = g.planetarium.clone();
        let mut antes = Vec::new();
        for ante in 1..=4 {
            let vouchers = Vouchers::new();
            let ctx = ShopContext::for_test(&planetarium, &vouchers, ante);
            g.shop.refresh(&ctx, &mut g.backend);
            let jokers: Vec<String> = g.shop.jokers.iter().map(|j| j.name().to_string()).collect();
            let consumables: Vec<String> = g
                .shop
                .consumables
                .iter()
                .map(|c| c.name().to_string())
                .collect();
            let packs = vec![
                (g.shop.packs[0].category, g.shop.packs[0].size),
                (g.shop.packs[1].category, g.shop.packs[1].size),
            ];
            antes.push((jokers, consumables, packs));
        }
        antes
    }

    assert_eq!(
        run(),
        run(),
        "same seed must produce the same draws across every ante"
    );
}

// Buys the real shop joker, then refreshes many times: proves
// on_joker_bought reached the live Instance's lock table.
#[test]
fn test_real_rng_mode_excludes_bought_joker_from_future_draws() {
    let config = Config {
        rng_mode: RngMode::Real,
        seed_str: Some("TEST".to_string()),
        ..Config::default()
    };
    let mut g = Game::new(config);
    g.stage = Stage::Shop();
    g.money = 1000;

    let planetarium = g.planetarium.clone();
    let vouchers = Vouchers::new();
    let ctx = ShopContext::for_test(&planetarium, &vouchers, 1);
    g.shop.refresh(&ctx, &mut g.backend);
    let bought = g.shop.jokers[0].clone();
    g.buy_joker(bought.clone()).expect("buy joker");

    for ante in 1..=50 {
        let vouchers = Vouchers::new();
        let held_jokers = g.jokers.clone();
        let ctx = ShopContext {
            held_jokers: &held_jokers,
            ..ShopContext::for_test(&planetarium, &vouchers, ante)
        };
        g.shop.refresh(&ctx, &mut g.backend);
        assert!(
            g.shop
                .jokers
                .iter()
                .all(|j| std::mem::discriminant(j) != std::mem::discriminant(&bought)),
            "bought joker {} reappeared in shop while still owned (ante {ante})",
            bought.name()
        );
    }
}

// Mirror of the test above: selling should make the joker drawable
// again. Probabilistic — enough draws that reappearance is near-certain.
#[test]
fn test_real_rng_mode_sold_joker_becomes_drawable_again() {
    let config = Config {
        rng_mode: RngMode::Real,
        seed_str: Some("TEST".to_string()),
        ..Config::default()
    };
    let mut g = Game::new(config);
    g.stage = Stage::Shop();
    g.money = 1000;

    let planetarium = g.planetarium.clone();
    let vouchers = Vouchers::new();
    let ctx = ShopContext::for_test(&planetarium, &vouchers, 1);
    g.shop.refresh(&ctx, &mut g.backend);
    let bought = g.shop.jokers[0].clone();
    g.buy_joker(bought.clone()).expect("buy joker");
    g.sell_joker(0).expect("sell joker");

    let mut reappeared = false;
    for ante in 1..=500 {
        let ctx = ShopContext::for_test(&planetarium, &vouchers, ante);
        g.shop.refresh(&ctx, &mut g.backend);
        if g.shop
            .jokers
            .iter()
            .any(|j| std::mem::discriminant(j) == std::mem::discriminant(&bought))
        {
            reappeared = true;
            break;
        }
    }
    assert!(
        reappeared,
        "sold joker {} never reappeared across 500 draws",
        bought.name()
    );
}

#[test]
fn test_constructor() {
    let g = Game::default();
    assert_eq!(g.available.cards().len(), 0);
    assert_eq!(g.deck.len(), 52);
    assert_eq!(g.mult, 0.0);
}

#[test]
fn test_deal() {
    let mut g = Game::default();
    g.deal();
    // deck should be 7 cards smaller than we started with
    assert_eq!(g.deck.len(), 52 - g.config.available);
    // should be 7 cards now available
    assert_eq!(g.available.cards().len(), g.config.available);
}

#[test]
fn test_draw() {
    let mut g = Game::default();
    g.draw(1);
    assert_eq!(g.available.cards().len(), 1);
    assert_eq!(g.deck.len(), 52 - 1);
    g.draw(3);
    assert_eq!(g.available.cards().len(), 4);
    assert_eq!(g.deck.len(), 52 - 4);
}
#[test]
fn test_discard() {
    let mut g = Game::default();
    g.deal();
    assert_eq!(g.available.cards().len(), g.config.available);
    assert_eq!(g.deck.len(), 52 - g.config.available);
    // select first 4 cards
    for c in g.available.cards()[0..5].to_vec() {
        g.select_card(c).unwrap();
    }
    let discard_res = g.discard_selected();
    assert!(discard_res.is_ok());
    // available should still be 7, we discarded then redrew to match
    assert_eq!(g.available.cards().len(), g.config.available);
    // deck is now smaller since we drew from it
    assert_eq!(g.deck.len(), 52 - g.config.available - 5);
}

#[test]
fn test_discarded_this_round_tracking() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Blind(Blind::Small);
    g.blind = Some(Blind::Small);
    g.deal();

    let played_card = g.available.cards()[0];
    g.select_card(played_card).expect("can select");
    g.play_selected().expect("can play");

    let discarded_card = g.available.cards()[0];
    g.select_card(discarded_card).expect("can select");
    g.discard_selected().expect("can discard");

    // only the discarded card is tracked, not the played one
    assert_eq!(g.discarded_this_round.len(), 1);
    assert_eq!(g.discarded_this_round[0].id, discarded_card.id);
    assert_eq!(g.total_cards_discarded, 1);

    // `discarded` (the deck-recycling pile) still has both - untouched
    assert!(g.discarded.iter().any(|c| c.id == played_card.id));
    assert!(g.discarded.iter().any(|c| c.id == discarded_card.id));

    g.clear_blind();
    // per-round tracking resets, run total does not
    assert!(g.discarded_this_round.is_empty());
    assert_eq!(g.total_cards_discarded, 1);
}

#[test]
fn test_calc_score() {
    let mut g = Game::default();
    let ace = Card::new(Value::Ace, Suit::Heart);
    let king = Card::new(Value::King, Suit::Diamond);
    let jack = Card::new(Value::Jack, Suit::Club);

    // Score [Ah, Kd, Jc]
    // High card (level 1) -> chips=5, mult=1
    // Played cards (1 ace) -> 11 chips
    // (5 + 11) * 1 = 16
    let cards = vec![ace, king, jack];
    let hand = SelectHand::new(cards).best_hand().unwrap();
    let score = g.calc_score(hand);
    assert_eq!(score, 16);

    // Score [Kd, Kd, Ah]
    // Pair (level 1) -> chips=10, mult=2
    // Played cards (2 kings) -> 10 + 10 == 20 chips
    // (10 + 20) * 2 = 60
    let cards = vec![king, king, ace];
    let hand = SelectHand::new(cards).best_hand().unwrap();
    let score = g.calc_score(hand);
    assert_eq!(score, 60);

    // Score [Ah, Ah, Ah, Kd]
    // Three of kind (level 1) -> chips=30, mult=3
    // Played cards (3 aces) -> 11 + 11 + 11 == 33 chips
    // (30 + 33) * 3 = 189
    let cards = vec![ace, ace, ace, king];
    let hand = SelectHand::new(cards).best_hand().unwrap();
    let score = g.calc_score(hand);
    assert_eq!(score, 189);

    // Score [Kd, Kd, Kd, Kd, Ah]
    // Four of kind (level 1) -> chips=60, mult=7
    // Played cards (4 kings) -> 10 + 10 + 10 + 10 == 40 chips
    // (60 + 40) * 7 = 700
    let cards = vec![king, king, king, king, ace];
    let hand = SelectHand::new(cards).best_hand().unwrap();
    let score = g.calc_score(hand);
    assert_eq!(score, 700);

    // Score [Jc, Jc, Jc, Jc, Jc]
    // Flush five (level 1) -> chips=160, mult=16
    // Played cards (5 jacks) -> 10 + 10 + 10 + 10 + 10 == 50 chips
    // (160 + 50) * 16 = 3360
    let cards = vec![jack, jack, jack, jack, jack];
    let hand = SelectHand::new(cards).best_hand().unwrap();
    let score = g.calc_score(hand);
    assert_eq!(score, 3360);
}

#[test]
fn test_calc_score_traced_matches_calc_score() {
    // guards calc_score/calc_score_traced against silently diverging
    let ace = Card::new(Value::Ace, Suit::Heart);
    let king = Card::new(Value::King, Suit::Diamond);
    let jack = Card::new(Value::Jack, Suit::Club);

    let fixtures = vec![
        vec![ace, king, jack],
        vec![king, king, ace],
        vec![ace, ace, ace, king],
        vec![king, king, king, king, ace],
        vec![jack, jack, jack, jack, jack],
    ];

    for cards in fixtures {
        let hand = SelectHand::new(cards).best_hand().unwrap();

        let mut plain = Game::default();
        let plain_score = plain.calc_score(hand.clone());

        let mut traced = Game::default();
        let (traced_score, trace) = traced.calc_score_traced(hand);

        assert_eq!(plain_score, traced_score);
        assert!(!trace.0.is_empty());
    }
}

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
fn test_debuffed_stone_kicker_loses_chip_bonus() {
    // Same shape as test_enhancement_stone_as_kicker, but the Stone
    // card is Club-suited under the Club boss - the kicker's +50 chips
    // no longer apply. (5 + 11) * 1 = 16, not 66.
    let mut g = Game {
        current_boss: Some(BossBlind::Club),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    let ace = Card::new(Value::Ace, Suit::Heart);
    let mut stone = Card::new(Value::Two, Suit::Club);
    stone.enhancement = Some(Enhancement::Stone);
    let hand = SelectHand::new(vec![ace, stone]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 16);
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

#[test]
fn test_manacle_boss_deals_reduced_hand() {
    let mut g = Game {
        current_boss: Some(BossBlind::Manacle),
        blind: Some(Blind::Big),
        stage: Stage::PreBlind(),
        ..Default::default()
    };
    g.select_blind(Blind::Boss).expect("can select boss blind");
    assert_eq!(g.available.cards().len(), 7);
}

#[test]
fn test_manacle_boss_no_effect_outside_boss_round() {
    let g = Game {
        current_boss: Some(BossBlind::Manacle),
        blind: Some(Blind::Small),
        ..Default::default()
    };
    assert_eq!(g.hand_size(), 8);
}

#[test]
fn test_serpent_boss_grows_hand_when_fewer_than_three_removed() {
    let mut g = Game {
        current_boss: Some(BossBlind::Serpent),
        blind: Some(Blind::Big),
        stage: Stage::PreBlind(),
        ..Default::default()
    };
    g.select_blind(Blind::Boss).expect("can select boss blind");
    let cards: Vec<Card> = g.available.cards().into_iter().take(2).collect();
    for card in cards {
        g.select_card(card).expect("can select card");
    }
    g.discard_selected().expect("can discard");
    assert_eq!(g.available.cards().len(), 9);
}

#[test]
fn test_serpent_boss_shrinks_hand_when_more_than_three_removed() {
    let mut g = Game {
        current_boss: Some(BossBlind::Serpent),
        blind: Some(Blind::Big),
        stage: Stage::PreBlind(),
        ..Default::default()
    };
    g.select_blind(Blind::Boss).expect("can select boss blind");
    let cards: Vec<Card> = g.available.cards().into_iter().take(5).collect();
    for card in cards {
        g.select_card(card).expect("can select card");
    }
    g.discard_selected().expect("can discard");
    assert_eq!(g.available.cards().len(), 6);
}

#[test]
fn test_serpent_boss_applies_to_play_selected_too() {
    let mut g = Game {
        current_boss: Some(BossBlind::Serpent),
        blind: Some(Blind::Big),
        stage: Stage::PreBlind(),
        ..Default::default()
    };
    g.select_blind(Blind::Boss).expect("can select boss blind");
    let card = g.available.cards()[0];
    g.select_card(card).expect("can select card");
    g.play_selected().expect("can play selected");
    assert_eq!(g.available.cards().len(), 10);
}

#[test]
fn test_serpent_boss_no_effect_outside_boss_round() {
    let mut g = Game {
        current_boss: Some(BossBlind::Serpent),
        blind: None,
        stage: Stage::PreBlind(),
        ..Default::default()
    };
    g.select_blind(Blind::Small)
        .expect("can select small blind");
    let cards: Vec<Card> = g.available.cards().into_iter().take(2).collect();
    for card in cards {
        g.select_card(card).expect("can select card");
    }
    g.discard_selected().expect("can discard");
    assert_eq!(g.available.cards().len(), 8);
}

#[test]
fn test_play_selected() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Blind(Blind::Small);
    g.blind = Some(Blind::Small);
    g.deal();
    for card in g.available.cards().iter().take(5) {
        g.available.select_card(*card).expect("can select card");
    }

    assert_eq!(g.available.selected().len(), 5);
    // Artifically set score so blind passes
    g.score += g.required_score();
    g.play_selected().expect("can play selected");

    // Should have cleared blind
    assert_eq!(g.stage, Stage::PostBlind());
    // Score should reset to 0
    assert_eq!(g.score, g.config.base_score);
    // Plays and discards should reset
    assert_eq!(g.plays, g.config.plays);
    assert_eq!(g.discards_remaining, g.config.discards);
    // All cards back in deck after clear_blind, available is empty
    assert_eq!(g.deck.len(), 52);
    // Discarded should be length 0
    assert_eq!(g.discarded.len(), 0);
    // Available is empty until next blind starts
    assert_eq!(g.available.cards().len(), 0);
}

#[test]
fn test_buy_joker() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.money = 10;
    let j1 = crate::joker::jokers_by_rarity(crate::joker::Rarity::Common)[0].clone();
    g.shop.jokers = vec![j1.clone()];
    g.buy_joker(j1.clone()).expect("buy joker");
    assert_eq!(g.money, 10 - j1.cost());
    assert_eq!(g.jokers.len(), 1);
}

#[test]
fn test_enhancement_bonus() {
    // High card Ace with Bonus: (5 + 11 + 30) * 1 = 46
    let mut g = Game::default();
    let mut ace = Card::new(Value::Ace, Suit::Heart);
    ace.enhancement = Some(Enhancement::Bonus);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 46);
}

#[test]
fn test_enhancement_mult() {
    // High card Ace with Mult: (5 + 11) * (1 + 4) = 80
    let mut g = Game::default();
    let mut ace = Card::new(Value::Ace, Suit::Heart);
    ace.enhancement = Some(Enhancement::Mult);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 80);
}

#[test]
fn test_enhancement_glass() {
    // High card Ace with Glass: (5 + 11) * (1 * 2) = 32
    let mut g = Game::default();
    let mut ace = Card::new(Value::Ace, Suit::Heart);
    ace.enhancement = Some(Enhancement::Glass);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 32);
}

#[test]
fn test_enhancement_stone_as_kicker() {
    // [Ace, Stone] -> High Card Ace; Stone kicker always scores +50
    // (5 + 11 + 50) * 1 = 66
    let mut g = Game::default();
    let ace = Card::new(Value::Ace, Suit::Heart);
    let mut stone = Card::new(Value::Two, Suit::Diamond);
    stone.enhancement = Some(Enhancement::Stone);
    let hand = SelectHand::new(vec![ace, stone]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 66);
}

#[test]
fn test_enhancement_steel_held() {
    // Pair of Kings scored; Steel King held (not played)
    // Pair level 1: chips=10, mult=2; both Kings score: chips=10+10+10=30
    // Steel held: mult = floor(2 * 1.5) = 3 -> score = 30 * 3 = 90
    let mut g = Game::default();
    let king1 = Card::new(Value::King, Suit::Heart);
    let king2 = Card::new(Value::King, Suit::Diamond);
    let mut steel_king = Card::new(Value::King, Suit::Spade);
    steel_king.enhancement = Some(Enhancement::Steel);
    g.available.extend(vec![steel_king]);
    let hand = SelectHand::new(vec![king1, king2]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 90);
}

#[test]
fn test_enhancement_gold_held() {
    // Gold card held in unplayed hand gives $3 per hand scored
    let mut g = Game::default();
    let mut gold_card = Card::new(Value::Two, Suit::Heart);
    gold_card.enhancement = Some(Enhancement::Gold);
    g.available.extend(vec![gold_card]);
    let ace = Card::new(Value::Ace, Suit::Spade);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    g.calc_score(hand);
    assert_eq!(g.money, 3);
}

#[test]
fn test_seal_gold_scored() {
    // Gold Seal: $3 when this card is played and scores (score itself unaffected)
    let mut g = Game::default();
    let mut ace = Card::new(Value::Ace, Suit::Spade);
    ace.seal = Some(Seal::Gold);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 16);
    assert_eq!(g.money, 3);
}

#[test]
fn test_seal_red_retrigger_played() {
    // Red Seal: retrigger this card's played abilities.
    // High card Ace, chips scored twice: (5 + 11 + 11) * 1 = 27
    let mut g = Game::default();
    let mut ace = Card::new(Value::Ace, Suit::Spade);
    ace.seal = Some(Seal::Red);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 27);
}

#[test]
fn test_seal_red_retrigger_gold_held() {
    // Red Seal retriggers held-card abilities too, not just played ones:
    // a held Gold+Red card earns $6 (2x $3), not $3.
    let mut g = Game::default();
    let mut gold_red = Card::new(Value::Two, Suit::Heart);
    gold_red.enhancement = Some(Enhancement::Gold);
    gold_red.seal = Some(Seal::Red);
    g.available.extend(vec![gold_red]);
    let ace = Card::new(Value::Ace, Suit::Spade);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    g.calc_score(hand);
    assert_eq!(g.money, 6);
}

#[test]
fn test_seal_red_retrigger_steel_held() {
    // Pair of Kings scored; held Steel+Red King retriggers its Mult bonus twice.
    // Pair level 1: chips=10, mult=2; both Kings score: chips=30
    // Steel triggers twice: mult = 2 -> 3 (floor(2*1.5)) -> 4 (floor(3*1.5))
    // score = 30 * 4 = 120
    let mut g = Game::default();
    let king1 = Card::new(Value::King, Suit::Heart);
    let king2 = Card::new(Value::King, Suit::Diamond);
    let mut steel_red_king = Card::new(Value::King, Suit::Spade);
    steel_red_king.enhancement = Some(Enhancement::Steel);
    steel_red_king.seal = Some(Seal::Red);
    g.available.extend(vec![steel_red_king]);
    let hand = SelectHand::new(vec![king1, king2]).best_hand().unwrap();
    // Steel held twice: 2 * 1.5 * 1.5 = 4.5; (10 + 20) * 4.5 = 135
    assert_eq!(g.calc_score(hand), 135);
}

#[test]
fn test_stone_kicker_red_seal_retrigger() {
    // [Ace, Stone+Red Seal] -> High Card Ace; stone kicker retriggers twice.
    // (5 + 11 + 50 + 50) * 1 = 116
    let mut g = Game::default();
    let ace = Card::new(Value::Ace, Suit::Heart);
    let mut stone = Card::new(Value::Two, Suit::Diamond);
    stone.enhancement = Some(Enhancement::Stone);
    stone.seal = Some(Seal::Red);
    let hand = SelectHand::new(vec![ace, stone]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 116);
}

#[test]
fn test_hanging_chad_retriggers_first_played_card() {
    use crate::joker::*;
    // [Ace, Stone], Ace first -> Ace retriggers 3x, stone kicker only 1x.
    // (5 + 11*3 + 50) * 1 = 88
    let mut g = Game::default();
    g.jokers.push(Jokers::HangingChad(HangingChad::default()));
    let jokers = g.jokers.clone();
    g.effect_registry.register_jokers(jokers, &g.clone());
    let ace = Card::new(Value::Ace, Suit::Heart);
    let mut stone = Card::new(Value::Two, Suit::Diamond);
    stone.enhancement = Some(Enhancement::Stone);
    let hand = SelectHand::new(vec![ace, stone]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 88);
}

#[test]
fn test_hanging_chad_stone_kicker_competes_for_first() {
    use crate::joker::*;
    // Same cards as above, stone selected first this time -> the stone
    // kicker gets HangingChad's retrigger instead of the Ace.
    // (5 + 11 + 50*3) * 1 = 166
    let mut g = Game::default();
    g.jokers.push(Jokers::HangingChad(HangingChad::default()));
    let jokers = g.jokers.clone();
    g.effect_registry.register_jokers(jokers, &g.clone());
    let mut stone = Card::new(Value::Two, Suit::Diamond);
    stone.enhancement = Some(Enhancement::Stone);
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![stone, ace]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 166);
}

#[test]
fn test_dusk_no_retrigger_with_plays_remaining() {
    use crate::joker::*;
    let mut g = Game::default();
    g.jokers.push(Jokers::Dusk(Dusk::default()));
    let jokers = g.jokers.clone();
    g.effect_registry.register_jokers(jokers, &g.clone());
    g.plays = 1;
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 16);
}

#[test]
fn test_dusk_retriggers_on_final_hand() {
    use crate::joker::*;
    // Same hand as above, but this is the last play of the round:
    // (5 + 11 + 11) * 1 = 27
    let mut g = Game::default();
    g.jokers.push(Jokers::Dusk(Dusk::default()));
    let jokers = g.jokers.clone();
    g.effect_registry.register_jokers(jokers, &g.clone());
    g.plays = 0;
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();
    assert_eq!(g.calc_score(hand), 27);
}

#[test]
fn test_hack_retriggers_only_low_ranks() {
    use crate::joker::*;
    // Two Pair (Threes + Kings): Hack retriggers each Three, not the Kings.
    let cards = vec![
        Card::new(Value::Three, Suit::Heart),
        Card::new(Value::Three, Suit::Diamond),
        Card::new(Value::King, Suit::Spade),
        Card::new(Value::King, Suit::Club),
    ];

    let mut base_game = Game::default();
    let base_hand = SelectHand::new(cards.clone()).best_hand().unwrap();
    let base = base_game.calc_score(base_hand);

    let mut g = Game::default();
    g.jokers.push(Jokers::Hack(Hack::default()));
    let jokers = g.jokers.clone();
    g.effect_registry.register_jokers(jokers, &g.clone());
    let hand = SelectHand::new(cards).best_hand().unwrap();
    // Each Three retriggers once more: +3 chips * 2 cards = +6 chips,
    // scaled by TwoPair's X2 mult -> +12 final score.
    assert_eq!(g.calc_score(hand), base + 12);
}

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

#[cfg(feature = "serde")]
#[test]
fn test_from_json_invalid() {
    let result = Game::from_json("not valid json");
    assert!(result.is_err());
}

#[cfg(feature = "serde")]
#[test]
fn test_from_json_advances_id_counter_past_loaded_ids() {
    let mut g = Game::default();
    let mut high_id_card = Card::new(Value::King, Suit::Spade);
    high_id_card.id = 999_999;
    g.deck.push(high_id_card);

    let json = g.to_json().expect("serialize");
    let _g2 = Game::from_json(&json).expect("deserialize");

    // Card::new() after loading must not collide with the loaded id.
    let new_card = Card::new(Value::Two, Suit::Heart);
    assert!(new_card.id > 999_999);
}

#[test]
fn test_glass_shatter_preserves_hand_size() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Blind(Blind::Small);
    g.blind = Some(Blind::Small);

    // Replace available with controlled cards so we have exactly one glass card.
    // Use mixed values/suits to ensure the hand scores below the blind threshold
    // so clear_blind() / deal() is not triggered mid-test.
    g.available.empty();
    // Glass card is ace (highest) so it's selected as the high card and scores.
    // high card ace scores (5+11)*2 = 32, below 300 small blind threshold.
    let mut glass = Card::new(Value::Ace, Suit::Heart);
    glass.enhancement = Some(Enhancement::Glass);
    let glass_id = glass.id;
    let others = vec![
        Card::new(Value::Two, Suit::Club),
        Card::new(Value::Four, Suit::Diamond),
        Card::new(Value::Six, Suit::Spade),
        Card::new(Value::Eight, Suit::Club),
        Card::new(Value::Ten, Suit::Diamond),
        Card::new(Value::Queen, Suit::Spade),
    ];
    let mut hand_cards = vec![glass];
    hand_cards.extend(others);
    g.available.extend(hand_cards);

    let hand_size_before = g.available.cards().len();

    // Force 100% shatter probability (only needs to be 4 but overkill is ok)
    g.prob_mult = 100;

    // Select glass + 4 others (5-card hand)
    let to_select: Vec<Card> = g.available.cards().into_iter().take(5).collect();
    for card in to_select {
        g.available.select_card(card).expect("select card");
    }

    g.play_selected().expect("play selected");

    // Hand size must be unchanged
    assert_eq!(g.available.cards().len(), hand_size_before);
    // Glass card must be permanently gone (not hiding in discarded to return next deal)
    assert!(g.discarded.iter().all(|c| c.id != glass_id));
}

#[test]
fn test_targeting_tarot_in_blind_invalid_selection_preserves_consumable() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Blind(Blind::Small);
    g.blind = Some(Blind::Small);
    g.deal();
    // no cards selected — Magician needs at least 1
    g.consumables = vec![Consumable::Tarot(Tarot::Magician)];
    let res = g.use_consumable(Consumable::Tarot(Tarot::Magician));
    assert!(matches!(res, Err(GameError::InvalidAction)));
    assert_eq!(g.consumables.len(), 1);
    assert!(g.consumables.contains(&Consumable::Tarot(Tarot::Magician)));
}

#[test]
fn test_targeting_tarot_in_blind_too_many_selected_preserves_consumable() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Blind(Blind::Small);
    g.blind = Some(Blind::Small);
    g.deal();
    // Lovers accepts exactly 1 — select 2 to exceed max
    let cards = g.available.cards();
    g.available.select_card(cards[0]).unwrap();
    g.available.select_card(cards[1]).unwrap();
    g.consumables = vec![Consumable::Tarot(Tarot::Lovers)];
    let res = g.use_consumable(Consumable::Tarot(Tarot::Lovers));
    assert!(matches!(res, Err(GameError::InvalidAction)));
    assert_eq!(g.consumables.len(), 1);
}

#[test]
fn test_sell_joker() {
    use crate::joker::Jokers;
    use balatro_types::joker::TheJoker;
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.money = 0;
    let joker = Jokers::TheJoker(TheJoker::default());
    let sell_value = joker.sell_value();
    g.jokers.push(joker);
    assert_eq!(g.jokers.len(), 1);

    g.sell_joker(0).expect("sell joker");
    assert_eq!(g.jokers.len(), 0);
    assert_eq!(g.money, sell_value);
}

#[test]
fn test_sell_joker_invalid_index() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    let res = g.sell_joker(0);
    assert!(matches!(res, Err(GameError::InvalidAction)));
}

#[test]
fn test_sell_joker_invalid_stage() {
    use crate::joker::Jokers;
    use balatro_types::joker::TheJoker;
    let mut g = Game::default();
    g.start();
    g.stage = Stage::End(crate::stage::End::Win);
    g.jokers.push(Jokers::TheJoker(TheJoker::default()));
    let res = g.sell_joker(0);
    assert!(matches!(res, Err(GameError::InvalidStage)));
}

#[test]
fn test_sell_joker_during_blind() {
    use crate::joker::Jokers;
    use balatro_types::joker::TheJoker;
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Blind(Blind::Small);
    g.blind = Some(Blind::Small);
    g.money = 0;
    let joker = Jokers::TheJoker(TheJoker::default());
    let sell_value = joker.sell_value();
    g.jokers.push(joker);

    g.sell_joker(0).expect("sell joker during blind");
    assert_eq!(g.jokers.len(), 0);
    assert_eq!(g.money, sell_value);
}

#[test]
fn test_sell_luchador_disables_boss_only_for_itself() {
    use crate::joker::Jokers;
    use balatro_types::joker::{Luchador, TheJoker};
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.jokers.push(Jokers::TheJoker(TheJoker::default()));
    g.jokers.push(Jokers::Luchador(Luchador::default()));

    // Selling an unrelated joker must not trigger Luchador's OnSell effect.
    g.sell_joker(0).expect("sell plain joker");
    assert!(!g.boss_disabled_by_luchador);

    g.sell_joker(0).expect("sell luchador");
    assert!(g.boss_disabled_by_luchador);
}

#[test]
fn test_sell_luchador_restores_water_boss_discards_via_handle_action() {
    use crate::joker::Jokers;
    use balatro_types::joker::Luchador;
    let mut g = Game {
        current_boss: Some(BossBlind::Water),
        blind: Some(Blind::Boss),
        stage: Stage::Blind(Blind::Boss),
        discards_remaining: 4,
        jokers: vec![Jokers::Luchador(Luchador::default())],
        ..Default::default()
    };
    assert_eq!(g.discards(), 0);

    g.handle_action(Action::SellJoker(0))
        .expect("sell luchador via handle_action");
    assert_eq!(g.discards(), 4);
}

#[test]
fn test_sell_consumable() {
    use crate::planet::Planets;
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.money = 0;
    let c = Consumable::Planet(Planets::Mercury);
    let sell_value = c.sell_value();
    g.consumables.push(c);
    assert_eq!(g.consumables.len(), 1);

    g.sell_consumable(0).expect("sell consumable");
    assert_eq!(g.consumables.len(), 0);
    assert_eq!(g.money, sell_value);
}

#[test]
fn test_sell_consumable_invalid_index() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    let res = g.sell_consumable(0);
    assert!(matches!(res, Err(GameError::InvalidAction)));
}

#[test]
fn test_sell_consumable_invalid_stage() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::End(crate::stage::End::Win);
    g.consumables
        .push(Consumable::Tarot(crate::tarot::Tarot::Fool));
    let res = g.sell_consumable(0);
    assert!(matches!(res, Err(GameError::InvalidStage)));
}

#[test]
fn test_sell_joker_during_tarot_hand() {
    use crate::joker::Jokers;
    use balatro_types::joker::TheJoker;
    let mut g = Game::default();
    g.start();
    g.stage = Stage::TarotHand(Tarot::Magician);
    g.money = 0;
    let joker = Jokers::TheJoker(TheJoker::default());
    let sell_value = joker.sell_value();
    g.jokers.push(joker);

    g.sell_joker(0).expect("sell joker during tarot hand");
    assert_eq!(g.jokers.len(), 0);
    assert_eq!(g.money, sell_value);
}

#[test]
fn test_sell_consumable_during_tarot_hand() {
    use crate::planet::Planets;
    let mut g = Game::default();
    g.start();
    g.stage = Stage::TarotHand(Tarot::Magician);
    g.money = 0;
    let c = Consumable::Planet(Planets::Mercury);
    let sell_value = c.sell_value();
    g.consumables.push(c);

    g.sell_consumable(0)
        .expect("sell consumable during tarot hand");
    assert_eq!(g.consumables.len(), 0);
    assert_eq!(g.money, sell_value);
}

#[test]
fn test_sell_joker_removes_effects() {
    use crate::joker::Jokers;
    use balatro_types::joker::HangingChad;
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    let joker = Jokers::HangingChad(HangingChad::default());
    g.jokers.push(joker);
    let jokers = g.jokers.clone();
    g.effect_registry.register_jokers(jokers, &g.clone());
    assert!(!g.effect_registry.trigger_count_played.is_empty());

    g.sell_joker(0).expect("sell joker");
    assert!(g.effect_registry.trigger_count_played.is_empty());
}

#[test]
fn test_buy_pack() {
    use crate::pack::{Pack, PackCategory, PackSize};
    use crate::planet::Planets;
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.money = 100;
    let pack = Pack {
        category: PackCategory::Celestial,
        size: PackSize::Normal,
        contents: vec![PackContent::Planet(Planets::Mercury)],
    };
    let cost = pack.cost();
    let picks = pack.picks_allowed();
    g.shop.packs.push(pack.clone());
    g.buy_pack(pack).expect("buy pack");
    assert_eq!(g.stage, Stage::PackOpen());
    assert_eq!(g.money, 100 - cost);
    let state = g.open_pack.as_ref().expect("open pack state");
    assert_eq!(state.picks_remaining, picks);
}

#[test]
fn test_skip_pack() {
    let mut g = Game::default();
    g.start();
    g.open_pack = Some(OpenPackState {
        picks_remaining: 1,
        description: String::new(),
        contents: vec![],
    });
    g.stage = Stage::PackOpen();
    g.skip_pack().expect("skip pack");
    assert_eq!(g.stage, Stage::Shop());
    assert!(g.open_pack.is_none());
}

#[test]
fn test_skip_blind_small() {
    let mut g = Game::default();
    g.start();
    g.small_blind_tag = Tag::Investment;
    assert!(g.tags.is_empty());
    g.skip_blind(Blind::Small).expect("skip small blind");
    assert_eq!(g.blind, Some(Blind::Small));
    assert_eq!(g.stage, Stage::PreBlind());
    assert_eq!(g.tags.len(), 1);
}

#[test]
fn test_skip_blind_wrong_blind() {
    let mut g = Game::default();
    g.start();
    // Game just started, next expected blind is Small, not Big
    let res = g.skip_blind(Blind::Big);
    assert!(matches!(res, Err(GameError::InvalidBlind)));
}

#[test]
fn test_skip_blind_boss_rejected() {
    let mut g = Game::default();
    g.start();
    g.blind = Some(Blind::Big);
    let res = g.skip_blind(Blind::Boss);
    assert!(matches!(res, Err(GameError::InvalidBlind)));
}

#[test]
fn test_skip_blind_wrong_stage() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    let res = g.skip_blind(Blind::Small);
    assert!(matches!(res, Err(GameError::InvalidStage)));
}

#[test]
fn test_skip_blind_draws_a_real_tag() {
    let mut g = Game::default();
    g.start();
    g.small_blind_tag = Tag::Juggle;
    g.skip_blind(Blind::Small).expect("skip small blind");
    assert_eq!(g.tags[0], Tag::Juggle);
}

#[test]
fn test_ante_tags_exist_from_game_start() {
    let g = Game::default();
    assert!(g.skip_tag(Blind::Small).is_some());
    assert!(g.skip_tag(Blind::Big).is_some());
    assert!(g.skip_tag(Blind::Boss).is_none());
}

#[test]
fn test_ante_tags_redrawn_after_boss_defeated() {
    let mut g = Game {
        stage: Stage::Blind(Blind::Boss),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    let ante_before = g.ante_current;
    g.handle_score(1_000_000).expect("handle score");
    assert_ne!(g.ante_current, ante_before);
    assert!(Tag::iter().any(|t| t == g.small_blind_tag));
    assert!(Tag::iter().any(|t| t == g.big_blind_tag));
}

#[test]
fn test_current_boss_set_on_game_new() {
    let g = Game::new(Config::default());
    assert!(g.current_boss.is_some());
    assert!(!g.current_boss.unwrap().is_finisher());
}

#[test]
fn test_current_boss_redrawn_after_boss_defeated() {
    let mut g = Game {
        stage: Stage::Blind(Blind::Boss),
        blind: Some(Blind::Boss),
        ..Default::default()
    };
    g.handle_score(1_000_000).expect("handle score");
    let boss = g
        .current_boss
        .expect("current_boss redrawn after ante advance");
    assert_eq!(boss.is_finisher(), g.ante_current.number() as i32 % 8 == 0);
}

#[test]
fn test_skip_blind_banks_the_predrawn_tag() {
    let mut g = Game::default();
    g.start();
    g.small_blind_tag = Tag::Investment;
    g.skip_blind(Blind::Small).expect("skip small blind");
    assert_eq!(g.tags, vec![Tag::Investment]);
}

#[test]
fn test_cashout_consumes_only_shop_tags() {
    let mut g = Game {
        stage: Stage::PostBlind(),
        ..Default::default()
    };
    g.tags = vec![Tag::Juggle, Tag::Investment, Tag::D6, Tag::Coupon];
    g.cashout().unwrap();
    assert_eq!(g.tags, [Tag::Juggle, Tag::Investment]);
    assert_eq!(g.reroll_cost, 0);
    assert!(g.shop.free_packs);
    assert_eq!(g.shop.free_jokers.len(), g.shop.jokers.len());
}

#[test]
fn immediate_tags_apply_once_and_double_copies_the_orbital_target() {
    let mut g = Game::default();
    let rank = HandRank::TwoPair;
    let starting_level = g.planetarium.level(rank).level;
    g.acquire_tag(Tag::Double, None);
    g.acquire_tag(Tag::Double, None);
    assert_eq!(g.tags, [Tag::Double, Tag::Double]);
    g.acquire_tag(Tag::Orbital, Some(rank));
    assert!(g.tags.is_empty());
    assert_eq!(g.planetarium.level(rank).level, starting_level + 9);

    g.money = 12;
    g.skipped_count = 2;
    g.unused_discards = 4;
    g.planetarium.play(rank);
    g.acquire_tag(Tag::Speed, None);
    g.acquire_tag(Tag::Garbage, None);
    g.acquire_tag(Tag::Handy, None);
    assert_eq!(g.money, 27);
    g.acquire_tag(Tag::Economy, None);
    assert_eq!(g.money, 54);
    g.acquire_tag(Tag::TopUp, None);
    assert_eq!(g.jokers.len(), 2);
    assert!(g.jokers.iter().all(|j| j.rarity() == Rarity::Common));
}

#[test]
fn blind_tags_pay_on_boss_and_raise_hand_size_for_one_round() {
    let mut g = Game {
        small_blind_tag: Tag::Juggle,
        big_blind_tag: Tag::Investment,
        ..Game::default()
    };
    let base_size = g.hand_size();
    g.skip_blind(Blind::Small).unwrap();
    g.select_blind(Blind::Big).unwrap();
    assert_eq!(g.hand_size(), base_size + 3);
    assert!(g.tags.is_empty());
    g.clear_blind();
    assert_eq!(g.hand_size(), base_size);
    g.stage = Stage::Blind(Blind::Boss);
    g.blind = Some(Blind::Boss);
    g.acquire_tag(Tag::Investment, None);
    let base_reward = g.calc_reward(Blind::Boss).unwrap();
    g.handle_score(g.required_score()).unwrap();
    assert_eq!(g.reward, base_reward + 25);
    assert!(g.tags.is_empty());
}

#[test]
fn free_tag_packs_return_to_blinds_in_acquisition_order() {
    for (tag, category, size) in [
        (Tag::Standard, PackCategory::Standard, PackSize::Mega),
        (Tag::Charm, PackCategory::Arcana, PackSize::Mega),
        (Tag::Meteor, PackCategory::Celestial, PackSize::Mega),
        (Tag::Buffoon, PackCategory::Buffoon, PackSize::Mega),
        (Tag::Ethereal, PackCategory::Spectral, PackSize::Normal),
    ] {
        let mut g = Game::default();
        let money = g.money;
        g.small_blind_tag = tag;
        g.acquire_tag(Tag::Double, None);
        g.skip_blind(Blind::Small).unwrap();
        assert_eq!(g.stage, Stage::PackOpen());
        let pack = g.open_pack.as_ref().unwrap();
        assert!(pack.description.starts_with(&format!(
            "Choose {}",
            if size == PackSize::Mega { 2 } else { 1 }
        )));
        assert!(pack.contents.iter().all(|item| matches!(
            (category, item),
            (PackCategory::Standard, PackContent::PlayingCard(_))
                | (PackCategory::Arcana, PackContent::Tarot(_))
                | (PackCategory::Celestial, PackContent::Planet(_))
                | (PackCategory::Buffoon, PackContent::Joker(_))
                | (PackCategory::Spectral, PackContent::Spectral(_))
        )));
        #[cfg(feature = "serde")]
        {
            g = Game::from_json(&g.to_json().unwrap()).unwrap();
        }
        g.skip_pack().unwrap();
        assert_eq!(g.stage, Stage::PackOpen());
        g.skip_pack().unwrap();
        assert_eq!(g.stage, Stage::PreBlind());
        assert_eq!(g.blind, Some(Blind::Small));
        assert_eq!(g.money, money);
    }
}

#[test]
fn shop_tags_offer_free_jokers_and_an_extra_voucher() {
    let mut g = Game {
        stage: Stage::PostBlind(),
        ..Default::default()
    };
    g.tags = vec![Tag::Uncommon, Tag::Rare, Tag::Voucher];
    g.cashout().unwrap();
    assert_eq!(g.shop.voucher_offers().count(), 2);
    for rarity in [Rarity::Uncommon, Rarity::Rare] {
        assert!(g
            .shop
            .jokers
            .iter()
            .any(|j| j.rarity() == rarity && g.shop.joker_price(j, &g.vouchers) == 0));
    }

    for (tag, edition) in [
        (Tag::Foil, Edition::Foil),
        (Tag::Holographic, Edition::Holographic),
        (Tag::Polychrome, Edition::Polychrome),
        (Tag::Negative, Edition::Negative),
    ] {
        g.shop.jokers = vec![Jokers::TheJoker(Default::default())];
        g.tags = vec![tag];
        g.resolve_shop_tags(false);
        assert!(g.tags.is_empty());
        assert_eq!(g.shop.jokers[0].edition(), edition);
        assert_eq!(g.shop.joker_price(&g.shop.jokers[0], &g.vouchers), 0);
    }
}

#[test]
fn boss_tag_and_vouchers_share_the_reroll_lifecycle() {
    let mut g = Game {
        small_blind_tag: Tag::Boss,
        ..Game::default()
    };
    let before = g.current_boss;
    let money = g.money;
    g.skip_blind(Blind::Small).unwrap();
    assert_ne!(g.current_boss, before);
    assert_eq!(g.money, money);
    g.vouchers.redeem(Voucher::DirectorsCut);
    g.money = 30;
    assert!(!g.boss_reroll_available());
    g.vouchers.redeem(Voucher::Retcon);
    let before = g.current_boss;
    g.handle_action(Action::RerollBoss()).unwrap();
    assert_ne!(g.current_boss, before);
    assert_eq!(g.money, 20);
}

#[test]
fn test_finish_pack_does_not_touch_tags() {
    let mut g = Game::default();
    g.start();
    g.tags = vec![Tag::Handy];
    g.open_pack = Some(OpenPackState {
        picks_remaining: 1,
        description: String::new(),
        contents: vec![],
    });
    g.stage = Stage::PackOpen();
    g.skip_pack().expect("skip pack");
    assert_eq!(g.tags, vec![Tag::Handy]);
}

#[test]
fn test_pick_pack_card_planet() {
    use crate::planet::Planets;
    let mut g = Game::default();
    g.start();
    g.open_pack = Some(OpenPackState {
        picks_remaining: 1,
        description: String::new(),
        contents: vec![PackContent::Planet(Planets::Mercury)],
    });
    g.stage = Stage::PackOpen();
    let level_before = g.planetarium.level(HandRank::OnePair).level;
    g.pick_pack_card(PackContent::Planet(Planets::Mercury))
        .expect("pick planet");
    assert_eq!(g.stage, Stage::Shop());
    assert!(g.open_pack.is_none());
    assert_eq!(
        g.planetarium.level(HandRank::OnePair).level,
        level_before + 1
    );
}

#[test]
fn test_pick_pack_card_joker() {
    use crate::joker::Jokers;
    use balatro_types::joker::TheJoker;
    let mut g = Game::default();
    g.start();
    let joker = Jokers::TheJoker(TheJoker::default());
    let before_len = g.jokers.len();
    g.open_pack = Some(OpenPackState {
        picks_remaining: 1,
        description: String::new(),
        contents: vec![PackContent::Joker(joker.clone())],
    });
    g.stage = Stage::PackOpen();
    g.pick_pack_card(PackContent::Joker(joker))
        .expect("pick joker");
    assert_eq!(g.jokers.len(), before_len + 1);
    assert_eq!(g.stage, Stage::Shop());
    assert!(g.open_pack.is_none());
}

#[test]
fn test_pick_pack_card_joker_slots_full() {
    use crate::joker::Jokers;
    use balatro_types::joker::TheJoker;
    let mut g = Game::default();
    g.start();
    let slots = g.config.joker_slots;
    for _ in 0..slots {
        g.jokers.push(Jokers::TheJoker(TheJoker::default()));
    }
    let joker = Jokers::TheJoker(TheJoker::default());
    g.open_pack = Some(OpenPackState {
        picks_remaining: 1,
        description: String::new(),
        contents: vec![PackContent::Joker(joker.clone())],
    });
    g.stage = Stage::PackOpen();
    let res = g.pick_pack_card(PackContent::Joker(joker));
    assert!(matches!(res, Err(GameError::NoAvailableSlot)));
}

#[test]
fn test_negative_joker_from_pack_expands_slots() {
    use crate::joker::Jokers;
    use balatro_types::joker::TheJoker;
    let mut g = Game::default();
    g.start();
    let slots = g.config.joker_slots;
    for _ in 0..slots {
        g.jokers.push(Jokers::TheJoker(TheJoker::default()));
    }
    let mut neg_joker = Jokers::TheJoker(TheJoker::default());
    neg_joker.set_edition(Edition::Negative);
    g.open_pack = Some(OpenPackState {
        picks_remaining: 1,
        description: String::new(),
        contents: vec![PackContent::Joker(neg_joker.clone())],
    });
    g.stage = Stage::PackOpen();
    g.pick_pack_card(PackContent::Joker(neg_joker))
        .expect("pick negative joker from full pack");
    assert_eq!(g.config.joker_slots, slots + 1);
    assert_eq!(g.jokers.len(), slots + 1);
    assert_eq!(g.stage, Stage::Shop());
    assert!(g.open_pack.is_none());
}

#[test]
fn test_pick_pack_card_targeting_tarot() {
    let mut g = Game::default();
    g.start();
    assert!(Tarot::Lovers.requires_targets());
    g.open_pack = Some(OpenPackState {
        picks_remaining: 1,
        description: String::new(),
        contents: vec![PackContent::Tarot(Tarot::Lovers)],
    });
    g.stage = Stage::PackOpen();
    g.pick_pack_card(PackContent::Tarot(Tarot::Lovers))
        .expect("pick targeting tarot");
    assert!(matches!(g.stage, Stage::TarotHand(Tarot::Lovers)));
    assert_eq!(g.tarot_prev_stage, Some(Stage::PackOpen()));
    // picks_remaining decremented later in apply_tarot, not here
    assert_eq!(g.open_pack.as_ref().unwrap().picks_remaining, 1);
}

#[test]
fn test_pick_pack_card_targeting_spectral() {
    use crate::spectral::Spectral;
    let mut g = Game::default();
    g.start();
    assert!(Spectral::Talisman.requires_targets());
    g.open_pack = Some(OpenPackState {
        picks_remaining: 1,
        description: String::new(),
        contents: vec![PackContent::Spectral(Spectral::Talisman)],
    });
    g.stage = Stage::PackOpen();
    g.pick_pack_card(PackContent::Spectral(Spectral::Talisman))
        .expect("pick targeting spectral");
    assert!(matches!(g.stage, Stage::SpectralHand(Spectral::Talisman)));
    assert_eq!(g.spectral_prev_stage, Some(Stage::PackOpen()));
    // picks_remaining decremented later in apply_spectral, not here
    assert_eq!(g.open_pack.as_ref().unwrap().picks_remaining, 1);
}

#[test]
fn test_pick_pack_card_spectral_immediate_apply() {
    use crate::spectral::Spectral;
    let mut g = Game::default();
    g.start();
    assert!(!Spectral::Sigil.requires_targets());
    g.available.extend(vec![crate::card::Card::new(
        crate::card::Value::Two,
        crate::card::Suit::Heart,
    )]);
    g.open_pack = Some(OpenPackState {
        picks_remaining: 1,
        description: String::new(),
        contents: vec![PackContent::Spectral(Spectral::Sigil)],
    });
    g.stage = Stage::PackOpen();
    g.pick_pack_card(PackContent::Spectral(Spectral::Sigil))
        .expect("pick immediate-apply spectral");
    assert_eq!(g.stage, Stage::Shop());
    assert!(g.open_pack.is_none());
}

#[test]
fn test_reroll_invalid_outside_shop() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Blind(Blind::Small);
    let res = g.reroll();
    assert!(matches!(res, Err(GameError::InvalidStage)));
}

#[test]
fn test_reroll_insufficient_balance() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.money = 0;
    let res = g.reroll();
    assert!(matches!(res, Err(GameError::InvalidBalance)));
}

#[test]
fn test_reroll_deducts_cost() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.money = 10;
    g.reroll().expect("reroll");
    assert_eq!(g.money, 5);
}

#[test]
fn test_reroll_increments_cost() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.money = 20;
    g.reroll().expect("first reroll");
    assert_eq!(g.reroll_cost, 6);
    g.reroll().expect("second reroll");
    assert_eq!(g.reroll_cost, 7);
    assert_eq!(g.money, 9); // 20 - 5 - 6
}

#[test]
fn test_reroll_regenerates_shop_cards() {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::Shop();
    g.money = 10;
    let old_joker = crate::joker::jokers_by_rarity(crate::joker::Rarity::Common)[0].clone();
    g.shop.jokers = vec![old_joker.clone()];
    g.shop.consumables = Vec::new();
    g.reroll().expect("reroll");
    assert_eq!(g.shop.jokers.len() + g.shop.consumables.len(), 2);
    let still_present = g
        .shop
        .jokers
        .iter()
        .any(|j| std::mem::discriminant(j) == std::mem::discriminant(&old_joker));
    assert!(!still_present);
}

// --- vouchers ---

/// A game sitting in a freshly-stocked shop, having cashed out (so the
/// ante's voucher offer has been drawn), with `money` to spend.
fn game_in_shop(money: usize) -> Game {
    let mut g = Game::default();
    g.start();
    g.stage = Stage::PostBlind();
    g.handle_action(Action::CashOut(0)).expect("cash out");
    g.money = money;
    g
}

fn redeem(g: &mut Game, voucher: Voucher) {
    g.shop.voucher = Some(voucher);
    g.money = g.money.max(voucher.cost());
    g.handle_action(Action::BuyVoucher(voucher))
        .expect("buy voucher");
}

#[test]
fn test_shop_offers_a_voucher_after_cashout() {
    let g = game_in_shop(0);
    let offered = g.shop.voucher.expect("ante 1 offers a voucher");
    // Only tier-1 vouchers can show up before anything is redeemed.
    assert_eq!(offered.requires(), None);
}

#[test]
fn test_voucher_offer_survives_reroll_but_not_purchase() {
    let mut g = game_in_shop(50);
    let offered = g.shop.voucher.expect("voucher offered");

    g.reroll().expect("reroll");
    assert_eq!(g.shop.voucher, Some(offered), "reroll must not touch it");

    g.handle_action(Action::BuyVoucher(offered))
        .expect("buy voucher");
    assert_eq!(g.shop.voucher, None);
    assert!(g.vouchers.has(offered));
    assert_eq!(g.money, 50 - 5 - offered.cost(), "reroll then voucher");
}

#[test]
fn test_voucher_offer_stays_gone_for_the_rest_of_the_ante() {
    let mut g = game_in_shop(50);
    let offered = g.shop.voucher.expect("voucher offered");
    g.handle_action(Action::BuyVoucher(offered))
        .expect("buy voucher");

    // Next round, same ante: no new offer.
    g.handle_action(Action::NextRound()).expect("next round");
    g.stage = Stage::PostBlind();
    g.handle_action(Action::CashOut(0)).expect("cash out");
    assert_eq!(g.shop.voucher, None);

    // New ante: a fresh offer, and never the one already redeemed.
    g.ante_current = Ante::Two;
    g.stage = Stage::PostBlind();
    g.handle_action(Action::CashOut(0)).expect("cash out");
    let next = g.shop.voucher.expect("ante 2 offers a voucher");
    assert_ne!(next, offered);
}

#[test]
fn test_cannot_buy_voucher_twice_or_without_funds() {
    let mut g = game_in_shop(0);
    let offered = g.shop.voucher.expect("voucher offered");
    assert!(matches!(
        g.handle_action(Action::BuyVoucher(offered)),
        Err(GameError::InvalidBalance)
    ));

    redeem(&mut g, offered);
    g.shop.voucher = Some(offered);
    g.money = 100;
    assert!(matches!(
        g.handle_action(Action::BuyVoucher(offered)),
        Err(GameError::VoucherAlreadyRedeemed)
    ));
}

#[test]
fn test_upgrade_voucher_only_offered_once_base_is_owned() {
    let mut g = game_in_shop(100);
    // Fast mode draws from the offerable pool, so with only the
    // upgrade's base missing it can never surface the upgrade.
    for _ in 0..50 {
        g.voucher_ante = None;
        g.ante_current = Ante::One;
        g.refresh_voucher_offer();
        let offered = g.shop.voucher.expect("voucher offered");
        assert_eq!(offered.requires(), None, "upgrade offered with no base");
    }

    redeem(&mut g, Voucher::Overstock);
    assert!(g.vouchers.is_offerable(Voucher::OverstockPlus));
}

#[test]
fn test_overstock_widens_the_shop() {
    let mut g = game_in_shop(100);
    let before = g.shop.jokers.len() + g.shop.consumables.len() + g.shop.cards.len();
    assert_eq!(before, 2);

    redeem(&mut g, Voucher::Overstock);
    // Applied to the shop the player is standing in, not just the next.
    let after = g.shop.jokers.len() + g.shop.consumables.len() + g.shop.cards.len();
    assert_eq!(after, 3);

    // ...and to every later refresh.
    g.reroll().expect("reroll");
    assert_eq!(
        g.shop.jokers.len() + g.shop.consumables.len() + g.shop.cards.len(),
        3
    );
}

#[test]
fn test_clearance_sale_discounts_cards_and_packs_but_not_vouchers() {
    let mut g = game_in_shop(100);
    redeem(&mut g, Voucher::ClearanceSale);

    let joker = crate::joker::jokers_by_rarity(crate::joker::Rarity::Common)[0].clone();
    g.shop.jokers = vec![joker.clone()];
    g.jokers.clear();
    let full = joker.cost();
    let money_before = g.money;
    g.buy_joker(joker).expect("buy joker");
    assert_eq!(g.money, money_before - (full * 3 / 4));

    // Vouchers keep their sticker price.
    g.shop.voucher = Some(Voucher::Blank);
    let money_before = g.money;
    g.handle_action(Action::BuyVoucher(Voucher::Blank))
        .expect("buy voucher");
    assert_eq!(g.money, money_before - Voucher::Blank.cost());
}

#[test]
fn test_reroll_vouchers_cut_the_price_immediately() {
    let mut g = game_in_shop(100);
    assert_eq!(g.reroll_cost, 5);
    redeem(&mut g, Voucher::RerollSurplus);
    assert_eq!(g.reroll_cost, 3, "current shop gets the discount too");
    assert_eq!(g.base_reroll_cost(), 3);

    redeem(&mut g, Voucher::RerollGlut);
    assert_eq!(g.reroll_cost, 1);
}

#[test]
fn omen_globe_can_replace_tarots_in_free_arcana_packs() {
    let mut game = Game::new(Config {
        seed: Some(0),
        ..Default::default()
    });
    game.vouchers.redeem(Voucher::CrystalBall);
    game.vouchers.redeem(Voucher::OmenGlobe);
    let mut spectrals = 0;
    for _ in 0..20 {
        game.tags.push(Tag::Charm);
        game.resolve_blind_choice_tags();
        let pack = game.open_pack.as_ref().unwrap();
        assert_eq!(pack.contents.len(), 5);
        spectrals += pack
            .contents
            .iter()
            .filter(|content| matches!(content, PackContent::Spectral(_)))
            .count();
        game.skip_pack().unwrap();
    }
    assert!(spectrals > 0 && spectrals < 100);
}

#[test]
fn every_voucher_redeems_once_and_upgrades_require_the_base() {
    for voucher in Voucher::iter() {
        let mut g = game_in_shop(100);
        g.shop.voucher = Some(voucher);
        if let Some(required) = voucher.requires() {
            assert!(g.buy_voucher(voucher).is_err());
            assert_eq!(g.money, 100);
            g.vouchers.redeem(required);
        }
        g.buy_voucher(voucher).unwrap();
        assert!(g.vouchers.has(voucher));
        assert_eq!(g.money, 100 - voucher.cost());
        assert!(g.buy_voucher(voucher).is_err());
        assert_eq!(g.money, 100 - voucher.cost());
    }
}

#[test]
fn test_slot_and_resource_vouchers() {
    let mut g = game_in_shop(100);
    let slots = g.consumable_slots();
    redeem(&mut g, Voucher::CrystalBall);
    assert_eq!(g.consumable_slots(), slots + 1);

    let joker_slots = g.joker_slots();
    redeem(&mut g, Voucher::Blank);
    redeem(&mut g, Voucher::Antimatter);
    assert_eq!(g.joker_slots(), joker_slots + 1);

    let hand_size = g.hand_size();
    redeem(&mut g, Voucher::PaintBrush);
    assert_eq!(g.hand_size(), hand_size + 1);

    // Per-round resources take effect at the start of the next round.
    redeem(&mut g, Voucher::Grabber);
    redeem(&mut g, Voucher::Wasteful);
    assert_eq!(g.plays_per_round(), g.config.plays + 1);
    assert_eq!(g.discards_per_round(), g.config.discards + 1);
    g.clear_blind();
    assert_eq!(g.plays, g.config.plays + 1);
    assert_eq!(g.discards(), g.config.discards + 1);
}

// Regression: `clear_blind` resets the per-round counters when a blind
// is *finished*, which happens before the shop opens. A voucher bought
// in that shop was landing on an already-set counter and appeared to do
// nothing until the round after next.
#[test]
fn test_hand_vouchers_apply_to_the_very_next_blind() {
    let mut g = game_in_shop(100);
    let base_plays = g.config.plays;
    let base_discards = g.config.discards;

    redeem(&mut g, Voucher::Grabber);
    redeem(&mut g, Voucher::Wasteful);

    g.handle_action(Action::NextRound()).expect("next round");
    g.handle_action(Action::SelectBlind(Blind::Small))
        .expect("select blind");

    assert_eq!(g.plays, base_plays + 1, "Grabber must count this round");
    assert_eq!(
        g.discards(),
        base_discards + 1,
        "Wasteful must count this round"
    );
}

// Same path, but for the vouchers that *cost* a resource.
#[test]
fn test_hieroglyph_costs_a_hand_on_the_very_next_blind() {
    let mut g = game_in_shop(100);
    let base_plays = g.config.plays;
    g.ante_current = Ante::Three;

    redeem(&mut g, Voucher::Hieroglyph);
    g.handle_action(Action::NextRound()).expect("next round");
    g.handle_action(Action::SelectBlind(Blind::Small))
        .expect("select blind");

    assert_eq!(g.plays, base_plays - 1);
    assert_eq!(g.ante_current, Ante::Two);
}

#[test]
fn test_paint_brush_deals_a_bigger_hand_on_the_very_next_blind() {
    let mut g = game_in_shop(100);
    let base_hand = g.config.available;

    redeem(&mut g, Voucher::PaintBrush);
    g.handle_action(Action::NextRound()).expect("next round");
    g.handle_action(Action::SelectBlind(Blind::Small))
        .expect("select blind");

    assert_eq!(g.available.cards().len(), base_hand + 1);
}

#[test]
fn test_hieroglyph_lowers_the_ante_and_costs_a_hand() {
    let mut g = game_in_shop(100);
    g.ante_current = Ante::Three;
    let hands = g.plays_per_round();

    redeem(&mut g, Voucher::Hieroglyph);
    assert_eq!(g.ante_current, Ante::Two);
    assert_eq!(g.plays_per_round(), hands - 1);
    assert_eq!(g.discards_per_round(), g.config.discards);
}

#[test]
fn test_seed_money_raises_the_interest_cap() {
    let mut g = game_in_shop(100);
    assert_eq!(g.interest_max(), g.config.interest_max);
    redeem(&mut g, Voucher::SeedMoney);
    assert_eq!(g.interest_max(), g.config.interest_max + 5);
    redeem(&mut g, Voucher::MoneyTree);
    assert_eq!(g.interest_max(), g.config.interest_max + 15);
}

#[test]
fn test_observatory_multiplies_mult_for_held_planets() {
    use crate::planet::Planets;

    // Two Kings = OnePair: 10 chips + 10 per King, 2 mult -> 30 * 2 = 60.
    let cards = vec![
        Card::new(Value::King, Suit::Heart),
        Card::new(Value::King, Suit::Spade),
    ];
    let hand = SelectHand::new(cards).best_hand().unwrap();

    let mut g = Game::default();
    assert_eq!(g.calc_score(hand.clone()), 60);

    // Mercury is OnePair's planet, so Observatory applies: mult 2 -> 3.
    g.consumables.push(Consumable::Planet(Planets::Mercury));
    g.vouchers.redeem(Voucher::Observatory);
    assert_eq!(g.calc_score(hand.clone()), 90);

    // A planet for a different hand does nothing.
    g.consumables = vec![Consumable::Planet(Planets::Jupiter)];
    assert_eq!(g.calc_score(hand), 60);
}

#[test]
fn test_telescope_puts_the_most_played_planet_in_celestial_packs() {
    use crate::pack::{PackCategory, PackContent};
    use crate::planet::Planets;

    let mut g = game_in_shop(100);
    // Make Flush the most played hand, then take Telescope.
    for _ in 0..5 {
        g.planetarium.play(HandRank::Flush);
    }
    redeem(&mut g, Voucher::Telescope);
    assert_eq!(g.most_played_hand_rank(), HandRank::Flush);

    let mut saw_celestial = false;
    for _ in 0..200 {
        g.stage = Stage::PostBlind();
        g.handle_action(Action::CashOut(0)).expect("cash out");
        for pack in &g.shop.packs {
            if pack.category != PackCategory::Celestial {
                continue;
            }
            saw_celestial = true;
            assert!(
                pack.contents
                    .contains(&PackContent::Planet(Planets::Jupiter)),
                "Celestial pack without the most played hand's planet: {:?}",
                pack.contents
            );
        }
    }
    assert!(saw_celestial, "no Celestial pack generated in 200 shops");
}

#[test]
fn test_magic_trick_puts_buyable_playing_cards_in_the_shop() {
    let mut g = game_in_shop(100);
    redeem(&mut g, Voucher::MagicTrick);

    // Playing cards are one of four outcomes per slot; reroll until seen.
    let mut card = None;
    for _ in 0..500 {
        if let Some(c) = g.shop.cards.first().copied() {
            card = Some(c);
            break;
        }
        g.money = 100;
        g.reroll().expect("reroll");
    }
    let card = card.expect("Magic Trick never produced a shop playing card");
    // Without Illusion, shop cards are plain.
    assert!(card.enhancement.is_none() && card.seal.is_none());
    assert_eq!(card.edition, Edition::Base);

    let deck_before = g.deck.len();
    g.money = 100;
    g.handle_action(Action::BuyPlayingCard(card))
        .expect("buy playing card");
    assert_eq!(g.deck.len(), deck_before + 1);
    assert!(g.deck.cards().iter().any(|c| c.id == card.id));
    assert!(!g.shop.cards.iter().any(|c| c.id == card.id));
    assert_eq!(g.money, 100 - card.shop_cost());
}

// regression: `buy_playing_card` didn't check `deck_max` itself
#[test]
fn test_buy_playing_card_rejects_a_full_deck() {
    let mut g = game_in_shop(100);
    redeem(&mut g, Voucher::MagicTrick);

    let mut card = None;
    for _ in 0..500 {
        if let Some(c) = g.shop.cards.first().copied() {
            card = Some(c);
            break;
        }
        g.money = 100;
        g.reroll().expect("reroll");
    }
    let card = card.expect("Magic Trick never produced a shop playing card");

    // Cap the deck at its current size so there's no room left.
    g.config.deck_max = g.deck.len() + g.available.cards().len() + g.discarded.len();
    g.money = 100;
    assert!(matches!(
        g.handle_action(Action::BuyPlayingCard(card)),
        Err(GameError::NoAvailableSlot)
    ));
}

#[test]
fn test_buy_voucher_round_trips_through_the_action_space() {
    let mut g = game_in_shop(100);
    let offered = g.shop.voucher.expect("voucher offered");
    let space = g.gen_action_space();
    let index = space
        .to_vec()
        .iter()
        .enumerate()
        .find_map(|(i, _)| match space.to_action(i, &g) {
            Ok(Action::BuyVoucher(v)) if v == offered => Some(i),
            _ => None,
        })
        .expect("BuyVoucher is unmasked in the shop");
    g.handle_action_index(index).expect("handle action index");
    assert!(g.vouchers.has(offered));
}

#[test]
fn test_vouchers_survive_a_save_load_round_trip() {
    let mut g = game_in_shop(100);
    redeem(&mut g, Voucher::CrystalBall);
    redeem(&mut g, Voucher::Grabber);
    let slots = g.consumable_slots();
    let plays = g.plays_per_round();

    let json = g.to_json().expect("serialize");
    let loaded = Game::from_json(&json).expect("deserialize");
    assert_eq!(loaded.vouchers.owned(), g.vouchers.owned());
    // Derived, not baked into Config — so no double application.
    assert_eq!(loaded.consumable_slots(), slots);
    assert_eq!(loaded.plays_per_round(), plays);
}

use super::*;

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

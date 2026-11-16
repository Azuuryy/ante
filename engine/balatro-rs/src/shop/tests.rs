use super::*;
use crate::rng::FastBackend;
use rand_chacha::ChaCha8Rng;

fn fast_backend() -> Backend {
    Backend::Fast(FastBackend::new(ChaCha8Rng::from_entropy()))
}

#[test]
fn test_shop_refresh() {
    let mut shop = Shop::new();
    let planetarium = Planetarium::new();
    assert_eq!(shop.jokers.len(), 0);
    assert_eq!(shop.packs.len(), 0);
    let vouchers = Vouchers::new();
    let ctx = ShopContext::for_test(&planetarium, &vouchers, 1);
    shop.refresh(&ctx, &mut fast_backend());
    assert_eq!(shop.jokers.len() + shop.consumables.len(), 2);
    assert_eq!(shop.packs.len(), 2);
}

#[test]
fn test_shop_buy_joker() {
    let mut shop = Shop::new();
    let j1 = jokers_by_rarity(Rarity::Common)[0].clone();
    shop.jokers = vec![j1.clone()];
    assert_eq!(shop.joker_from_index(0).expect("first joker"), j1.clone());
    shop.buy_joker(&j1).expect("buy joker");
}

#[test]
fn test_shop_buy_joker_disambiguates_duplicates_by_instance_id() {
    // Two copies of the same joker variant, distinguishable only by
    // instance_id (same as real shop generation would produce). Without
    // instance-id-sensitive equality, `buy_joker`'s `.position()` lookup
    // can't tell them apart and may remove the wrong one.
    let mut j1 = jokers_by_rarity(Rarity::Common)[0].clone();
    let mut j2 = j1.clone();
    j1.set_instance_id(1);
    j2.set_instance_id(2);

    let mut shop = Shop::new();
    shop.jokers = vec![j1.clone(), j2.clone()];

    let bought = shop.buy_joker(&j2).expect("buy joker");
    assert_eq!(bought.instance_id(), 2);
    assert_eq!(shop.jokers.len(), 1);
    assert_eq!(shop.jokers[0].instance_id(), 1);
}

#[test]
fn test_shop_buy_consumable() {
    let mut shop = Shop::new();
    shop.consumables = vec![Consumable::Planet(Planets::Mercury)];
    assert_eq!(shop.consumables.len(), 1);
    let c1 = shop.consumables[0];
    shop.buy_consumable(&c1).expect("buy consumable");
    assert_eq!(shop.consumables.len(), 0);
}

#[test]
fn test_shop_buy_pack() {
    let mut shop = Shop::new();
    let planetarium = Planetarium::new();
    let vouchers = Vouchers::new();
    let ctx = ShopContext::for_test(&planetarium, &vouchers, 1);
    shop.refresh(&ctx, &mut fast_backend());
    assert_eq!(shop.packs.len(), 2);
    let p1 = shop.packs[0].clone();
    let bought = shop.buy_pack(&p1).expect("buy pack");
    assert_eq!(bought.category, p1.category);
    assert_eq!(shop.packs.len(), 1);
}

#[test]
fn test_secret_planet_gating() {
    let planetarium = Planetarium::new();
    let gen = ConsumableGenerator::new();
    for _ in 0..500 {
        let c = gen.gen_planet_consumable(&planetarium, &[], &mut rand::thread_rng());
        let Consumable::Planet(planet) = c else {
            continue;
        };
        assert!(
            !planet.is_secret(),
            "secret planet generated before discovery"
        );
    }
}

#[test]
fn test_gen_moves_buy_consumable_slots_full() {
    let mut shop = Shop::new();
    let planetarium = Planetarium::new();
    let vouchers = Vouchers::new();
    let ctx = ShopContext::for_test(&planetarium, &vouchers, 1);
    shop.refresh(&ctx, &mut fast_backend());
    // slots full (held == consumable_slots)
    let moves = shop.gen_moves_buy_consumable(100, 2, 2, &vouchers);
    assert!(moves.is_none());
}

#[test]
fn test_gen_moves_buy_consumable_no_funds() {
    let mut shop = Shop::new();
    let planetarium = Planetarium::new();
    let vouchers = Vouchers::new();
    let ctx = ShopContext::for_test(&planetarium, &vouchers, 1);
    shop.refresh(&ctx, &mut fast_backend());
    // 0 money can't afford any planet ($3)
    let moves: Option<Vec<Action>> = shop
        .gen_moves_buy_consumable(0, 2, 0, &vouchers)
        .map(|i| i.collect());
    assert!(moves.is_none_or(|v| v.is_empty()));
}

#[test]
fn test_pack_gen_produces_valid_packs() {
    let planetarium = Planetarium::new();
    let gen = PackGenerator {
        gros_michel_extinct: false,
    };
    for _ in 0..50 {
        let pack = gen.gen_pack(&planetarium, 1, None, &[], &mut rand::thread_rng());
        let expected_count = balatro_seed::pack_card_count(pack.category, pack.size) as usize;
        assert_eq!(pack.contents.len(), expected_count);
    }
}

#[test]
fn test_pack_gen_can_produce_valid_spectral_packs() {
    let planetarium = Planetarium::new();
    let gen = PackGenerator {
        gros_michel_extinct: false,
    };
    let mut saw_spectral = false;
    for _ in 0..200 {
        let pack = gen.gen_pack(&planetarium, 1, None, &[], &mut rand::thread_rng());
        if pack.category != PackCategory::Spectral {
            continue;
        }
        saw_spectral = true;
        let expected_count = balatro_seed::pack_card_count(pack.category, pack.size) as usize;
        assert_eq!(pack.contents.len(), expected_count);
        for content in &pack.contents {
            let PackContent::Spectral(s) = content else {
                panic!("Spectral pack produced non-Spectral content: {content:?}");
            };
            assert!(!s.is_rare(), "{s:?} should never appear in a normal draw");
        }
    }
    assert!(
        saw_spectral,
        "expected at least one Spectral pack in 200 draws"
    );
}

#[test]
fn test_gen_joker_rarity_distribution_roughly_matches_weights() {
    use rand::SeedableRng;
    let mut rng = ChaCha8Rng::seed_from_u64(1234);
    let gen = JokerGenerator::new();
    let n = 2000;
    let mut common = 0;
    let mut uncommon = 0;
    let mut rare = 0;
    for _ in 0..n {
        match gen.gen_joker(1, &[], &mut rng).rarity() {
            Rarity::Common => common += 1,
            Rarity::Uncommon => uncommon += 1,
            Rarity::Rare => rare += 1,
            Rarity::Legendary => panic!("gen_joker should never roll Legendary"),
        }
    }
    // Generous bands around the intended 70/25/5 split - wide enough to
    // never flake, tight enough to catch the roll being broken/reverted
    // to Common-only.
    assert!(
        (1200..1900).contains(&common),
        "expected ~70% Common, got {common}/{n}"
    );
    assert!(
        (300..800).contains(&uncommon),
        "expected ~25% Uncommon, got {uncommon}/{n}"
    );
    assert!(
        (10..250).contains(&rare),
        "expected ~5% Rare, got {rare}/{n}"
    );
}

#[test]
fn test_gen_edition_playing_cards_never_negative() {
    use rand::SeedableRng;
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    // prob_mult saturates every gen_ratio numerator at 1000/1000 (always true);
    // with allow_negative=false this should fall through to Polychrome instead.
    for _ in 0..500 {
        assert_ne!(gen_edition(400, &mut rng, false), Edition::Negative);
    }
}

#[test]
fn test_gen_edition_jokers_can_be_negative() {
    use rand::SeedableRng;
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    assert_eq!(gen_edition(400, &mut rng, true), Edition::Negative);
}

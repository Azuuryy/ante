use super::*;
use crate::joker::JokerEffects;

#[test]
fn fast_backend_draw_ante_tags_draws_two_valid_tags() {
    let mut backend = FastBackend::new(ChaCha8Rng::seed_from_u64(1));
    let (small, big) = backend.draw_ante_tags();
    assert!(Tag::iter().any(|t| t == small));
    assert!(Tag::iter().any(|t| t == big));
}

#[test]
fn fast_backend_draw_boss_matches_category() {
    let mut backend = FastBackend::new(ChaCha8Rng::seed_from_u64(1));
    for ante in [1, 7, 8, 9, 16] {
        let boss = backend.draw_boss(ante);
        assert_eq!(
            boss.is_finisher(),
            ante % 8 == 0,
            "ante {ante} drew {boss:?}"
        );
    }
}

#[test]
fn fast_backend_draw_boss_respects_min_ante() {
    let mut backend = FastBackend::new(ChaCha8Rng::seed_from_u64(1));
    for _ in 0..200 {
        let boss = backend.draw_boss(1);
        assert!(boss.min_ante() <= 1, "ante 1 drew {boss:?} (min_ante > 1)");
    }
}

#[test]
fn fast_backend_prob_roll_respects_ratio_edges() {
    let mut backend = FastBackend::new(ChaCha8Rng::seed_from_u64(1));
    // numerator 0 -> never; numerator == denominator -> always,
    // regardless of what's drawn.
    for _ in 0..20 {
        assert!(!backend.prob_roll(0, 10));
    }
    for _ in 0..20 {
        assert!(backend.prob_roll(10, 10));
    }
    // numerator > denominator gets clamped to denominator (always true),
    // not a `gen_ratio` panic.
    assert!(backend.prob_roll(50, 10));
}

#[test]
fn fast_backend_shuffle_deck_reorders_without_losing_cards() {
    let mut backend = FastBackend::new(ChaCha8Rng::seed_from_u64(1));
    let mut deck = crate::deck::Deck::default();
    let before = deck.cards();
    backend.shuffle_deck(&mut deck);
    let after = deck.cards();

    assert_eq!(before.len(), after.len());
    assert_ne!(before, after, "shuffle with a fixed seed should reorder");
    let mut sorted_before = before.clone();
    let mut sorted_after = after.clone();
    sorted_before.sort();
    sorted_after.sort();
    assert_eq!(
        sorted_before, sorted_after,
        "shuffle must permute, not lose or duplicate cards"
    );
}

// `RealBackend`'s new methods all fall back to `extra_rng` (no real
// `Instance` equivalent wired up yet, see each method's doc comment) -
// this is a smoke test that the fallback wiring is complete and doesn't
// panic, not a claim of real-Balatro accuracy.
#[test]
fn real_backend_fallback_methods_all_work() {
    let fast = FastBackend::new(ChaCha8Rng::seed_from_u64(1));
    let mut backend = RealBackend::new("TESTSEED", fast);

    let mut deck = crate::deck::Deck::default();
    let before = deck.cards();
    backend.shuffle_deck(&mut deck);
    assert_ne!(before, deck.cards());

    assert!(!backend.prob_roll(0, 10));
    assert!(backend.prob_roll(10, 10));

    let (small, big) = backend.draw_ante_tags();
    assert!(Tag::iter().any(|t| t == small));
    assert!(Tag::iter().any(|t| t == big));

    let planetarium = Planetarium::new();
    let _ = backend.roll_random_planet(&planetarium, &[]);
    let _ = backend.roll_random_tarot(&[]);

    let joker = backend.gen_joker(1, 1, &[]);
    assert!(joker.is_implemented());
}

#[test]
fn real_backend_draw_boss_matches_category() {
    let fast = FastBackend::new(ChaCha8Rng::seed_from_u64(1));
    let mut backend = RealBackend::new("TESTSEED", fast);
    for ante in [1, 7, 8, 9, 16] {
        let boss = backend.draw_boss(ante);
        assert_eq!(
            boss.is_finisher(),
            ante % 8 == 0,
            "ante {ante} drew {boss:?}"
        );
    }
}

#[test]
fn real_backend_set_showman_propagates() {
    let fast = FastBackend::new(ChaCha8Rng::seed_from_u64(1));
    let mut backend = RealBackend::new("TESTSEED", fast);
    assert!(!backend.instance.params.showman);
    backend.set_showman(true);
    assert!(backend.instance.params.showman);
    backend.set_showman(false);
    assert!(!backend.instance.params.showman);
}

// Structural check on gen_pack's typed plumbing across many draws.
#[test]
fn real_backend_gen_pack_contents_match_category_and_count() {
    let fast = FastBackend::new(ChaCha8Rng::seed_from_u64(1));
    let mut backend = RealBackend::new("TESTSEED", fast);
    let planetarium = Planetarium::new();
    let vouchers = Vouchers::new();
    let mut seen_categories: std::collections::HashSet<PackCategory> =
        std::collections::HashSet::new();

    for ante in 1..=300 {
        let ctx = ShopContext::for_test(&planetarium, &vouchers, ante);
        let pack = backend.gen_pack(&ctx, None);
        seen_categories.insert(pack.category);

        let expected_count = balatro_seed::pack_card_count(pack.category, pack.size);
        assert_eq!(
            pack.contents.len() as i32,
            expected_count,
            "ante {ante}: {:?}/{:?} should hold {expected_count} cards",
            pack.category,
            pack.size
        );

        for content in &pack.contents {
            let matches = match pack.category {
                PackCategory::Buffoon => matches!(content, PackContent::Joker(_)),
                PackCategory::Standard => matches!(content, PackContent::PlayingCard(_)),
                PackCategory::Spectral => matches!(content, PackContent::Spectral(_)),
                PackCategory::Arcana => {
                    matches!(content, PackContent::Tarot(_) | PackContent::Spectral(_))
                }
                PackCategory::Celestial => {
                    matches!(content, PackContent::Planet(_) | PackContent::Spectral(_))
                }
            };
            assert!(
                matches,
                "ante {ante}: {:?} pack produced unexpected content {content:?}",
                pack.category
            );
        }
    }

    assert_eq!(
        seen_categories.len(),
        5,
        "expected all 5 pack categories across 300 draws, saw {seen_categories:?}"
    );
}

// regression: could offer a tier-2 upgrade before its base was ever redeemed.
#[test]
fn real_backend_gen_voucher_never_offers_an_ungated_upgrade() {
    let fast = FastBackend::new(ChaCha8Rng::seed_from_u64(1));
    let mut backend = RealBackend::new("TESTSEED", fast);
    let vouchers = Vouchers::new();
    let mut saw_any = false;

    for ante in 1..=300 {
        if let Some(v) = backend.gen_voucher(ante, &vouchers) {
            saw_any = true;
            assert!(
                vouchers.is_offerable(v),
                "ante {ante}: offered {v:?} with its base not owned"
            );
        }
    }

    assert!(saw_any, "no voucher drawn across 300 antes");
}

// `seed_joker_with_id` is `RealBackend`'s equivalent mint chokepoint to
// `JokerGenerator::gen_joker` (Fast mode) - Castle/MailInRebate must
// come out with a selector already set via `RealBackend`'s own
// `extra_rng`, not left `None`. Calls it directly with constructed
// jokers rather than sampling real-seed shop/pack generation and hoping
// to land on these two specific jokers among real Balatro's full
// 150-joker pool, which isn't reliably hit in any bounded sample.
#[test]
fn real_backend_seed_joker_with_id_rolls_discard_selector() {
    use crate::joker::{Castle, MailInRebate};

    let fast = FastBackend::new(ChaCha8Rng::seed_from_u64(1));
    let mut backend = RealBackend::new("TESTSEED", fast);
    let castle = seed_joker_with_id(Jokers::Castle(Castle::default()), &mut backend.extra_rng);
    assert!(
        castle.state().selector.is_some(),
        "Castle minted with no selector"
    );
    let mail = seed_joker_with_id(
        Jokers::MailInRebate(MailInRebate::default()),
        &mut backend.extra_rng,
    );
    assert!(
        mail.state().selector.is_some(),
        "MailInRebate minted with no selector"
    );
}

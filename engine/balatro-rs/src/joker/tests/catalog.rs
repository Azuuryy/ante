use super::*;

// balatro_types::Jokers now defines all jokers but only a subset have
// `effects()` behavior implemented. Shop/pack generation
// must never offer joker that silently does nothing.
#[test]
fn test_implemented_joker_count() {
    // 74 before Lot 3, then 10 from batch 1, 12 from batch 2a, 7 from batch 2b, 7 from batch 3, 9 from batch 4a, 3 from batch 4b and 12 from batch 5a.
    let count = Jokers::iter().filter(|j| j.is_implemented()).count();
    let extra = if cfg!(feature = "ante-extra-jokers") {
        3
    } else {
        0
    };
    assert_eq!(count, 134 + extra);
}

#[test]
fn test_jokers_by_rarity_never_returns_unimplemented() {
    for rarity in [
        Rarity::Common,
        Rarity::Uncommon,
        Rarity::Rare,
        Rarity::Legendary,
    ] {
        for j in jokers_by_rarity(rarity) {
            assert!(
                j.is_implemented(),
                "jokers_by_rarity({rarity}) returned unimplemented joker {}",
                j.name()
            );
        }
    }
}

#[test]
fn test_shop_joker_generation_never_produces_unimplemented() {
    use rand::SeedableRng;
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(42);
    let gen = crate::shop::JokerGenerator::new();
    for _ in 0..500 {
        let joker = gen.gen_joker(1, &[], &mut rng);
        assert!(
            joker.is_implemented(),
            "shop generated unimplemented joker {}",
            joker.name()
        );
    }
}

#[test]
fn test_the_joker() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);

    // Score Ace high without joker
    // High card (level 1) -> 5 chips, 1 mult
    // Played cards (1 ace) -> 11 chips
    // (5 + 11) * (1) = 16
    let before = 16;
    // Score Ace high with the Joker
    // High card (level 1) -> 5 chips, 1 mult
    // Played cards (1 ace) -> 11 chips
    // Joker (The Joker) -> 4 mult
    // (5 + 11) * (1 + 4) = 80
    let after = 80;

    let j = Jokers::TheJoker(TheJoker::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_lusty_joker() {
    let ah = Card::new(Value::Ace, Suit::Heart);
    let ac = Card::new(Value::Ace, Suit::Club);
    let ad = Card::new(Value::Ace, Suit::Diamond);
    let hand = SelectHand::new(vec![ah, ah, ac, ad]);

    // Score 4ok without joker
    // 4ok (level 1) -> 60 chips, 7 mult
    // Played cards (4 ace) -> 44 chips
    // (60 + 44) * (7) = 728
    let before = 728;
    // Score 4ok (2 hearts) with joker
    // 4ok (level 1) -> 60 chips, 7 mult
    // Played cards (4 ace) -> 44 chips
    // joker w/ 2 hearts = +6 mult
    // (60 + 44) * (7 + 6) = 1352
    let after = 1352;

    let j = Jokers::LustyJoker(LustyJoker::default());
    score_before_after_joker(j, hand, before, after)
}

#[test]
fn test_greedy_joker() {
    let ah = Card::new(Value::Ace, Suit::Heart);
    let ad = Card::new(Value::Ace, Suit::Diamond);
    let hand = SelectHand::new(vec![ad, ad, ad, ah]);

    // Score 4ok without joker
    // 4ok (level 1) -> 60 chips, 7 mult
    // Played cards (4 ace) -> 44 chips
    // (60 + 44) * (7) = 728
    let before = 728;
    // Score 4ok (3 diamonds) with joker
    // 4ok (level 1) -> 60 chips, 7 mult
    // Played cards (4 ace) -> 44 chips
    // joker w/ 3 diamonds = +9 mult
    // (60 + 44) * (7 + 9) = 1664
    let after = 1664;

    let j = Jokers::GreedyJoker(GreedyJoker::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_wrathful_joker() {
    let asp = Card::new(Value::Ace, Suit::Spade);
    let ad = Card::new(Value::Ace, Suit::Diamond);
    let hand = SelectHand::new(vec![asp, ad, ad, ad]);

    // Score 4ok without joker
    // 4ok (level 1) -> 60 chips, 7 mult
    // Played cards (4 ace) -> 44 chips
    // (60 + 44) * (7) = 728
    let before = 728;
    // Score 4ok (1 spade) with joker
    // 4ok (level 1) -> 60 chips, 7 mult
    // Played cards (4 ace) -> 44 chips
    // joker w/ 1 spade = +3 mult
    // (60 + 44) * (7 + 3) = 1040
    let after = 1040;

    let j = Jokers::WrathfulJoker(WrathfulJoker::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_gluttonous_joker() {
    let ac = Card::new(Value::Ace, Suit::Club);
    let hand = SelectHand::new(vec![ac, ac, ac, ac]);

    // Score 4ok without joker
    // 4ok (level 1) -> 60 chips, 7 mult
    // Played cards (4 ace) -> 44 chips
    // (60 + 44) * (7) = 728
    let before = 728;
    // Score 4ok (4 clubs) with joker
    // 4ok (level 1) -> 60 chips, 7 mult
    // Played cards (4 ace) -> 44 chips
    // joker w/ 4 clubs = +12 mult
    // (60 + 44) * (7 + 12) = 1976
    let after = 1976;

    let j = Jokers::GluttonousJoker(GluttonousJoker::default());
    score_before_after_joker(j, hand, before, after)
}

#[test]
fn test_jolly_joker() {
    let ac = Card::new(Value::Ace, Suit::Club);
    let hand = SelectHand::new(vec![ac, ac, ac, ac]);

    // Score 4ok without joker
    // 4ok (level 1) -> 60 chips, 7 mult
    // Played cards (4 ace) -> 44 chips
    // (60 + 44) * (7) = 728
    let before = 728;
    // Score 4ok with joker
    // 4ok (level 1) -> 60 chips, 7 mult
    // Played cards (4 ace) -> 44 chips
    // joker w/ pair = +8 mult
    // (60 + 44) * (7 + 8) = 1560
    let after = 1560;

    let j = Jokers::JollyJoker(JollyJoker::default());
    score_before_after_joker(j, hand, before, after)
}

#[test]
fn test_zany_joker() {
    let ac = Card::new(Value::Ace, Suit::Club);
    let hand = SelectHand::new(vec![ac, ac, ac, ac]);

    // Score 4ok without joker
    // 4ok (level 1) -> 60 chips, 7 mult
    // Played cards (4 ace) -> 44 chips
    // (60 + 44) * (7) = 728
    let before = 728;
    // Score 4ok with joker
    // 4ok (level 1) -> 60 chips, 7 mult
    // Played cards (4 ace) -> 44 chips
    // joker w/ 3ok = +12 mult
    // (60 + 44) * (7 + 12) = 1976
    let after = 1976;

    let j = Jokers::ZanyJoker(ZanyJoker::default());
    score_before_after_joker(j, hand, before, after)
}

#[test]
fn test_mad_joker() {
    let ac = Card::new(Value::Ace, Suit::Club);
    let kc = Card::new(Value::King, Suit::Club);
    let hand = SelectHand::new(vec![ac, ac, kc, kc]);

    // Score two pair without joker
    // two pair (level 1) -> 20 chips, 2 mult
    // Played cards (2 ace, 2 king) -> 42 chips
    // (20 + 42) * (2) = 124
    let before = 124;
    let j = Jokers::MadJoker(MadJoker::default());
    // Score two pair with joker
    // two pair (level 1) -> 20 chips, 2 mult
    // Played cards (2 ace, 2 king) -> 42 chips
    // joker w/ two pair = +10 mult
    // (20 + 42) * (2 + 10) = 744
    let after = 744;

    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_crazy_joker() {
    let two = Card::new(Value::Two, Suit::Club);
    let three = Card::new(Value::Three, Suit::Club);
    let four = Card::new(Value::Four, Suit::Club);
    let five = Card::new(Value::Five, Suit::Club);
    let six = Card::new(Value::Six, Suit::Heart);
    let hand = SelectHand::new(vec![two, three, four, five, six]);

    // Score straight without joker
    // straight (level 1) -> 30 chips, 4 mult
    // Played cards (2, 3, 4, 5, 6) -> 20 chips
    // (20 + 30) * (4) = 200
    let before = 200;
    // Score straight with joker
    // straight (level 1) -> 30 chips, 4 mult
    // Played cards (2, 3, 4, 5, 6) -> 20 chips
    // joker w/ straight = +12 mult
    // (20 + 30) * (4 + 12) = 800
    let after = 800;

    let j = Jokers::CrazyJoker(CrazyJoker::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_droll_joker() {
    let two = Card::new(Value::Two, Suit::Club);
    let three = Card::new(Value::Three, Suit::Club);
    let four = Card::new(Value::Four, Suit::Club);
    let five = Card::new(Value::Five, Suit::Club);
    let ten = Card::new(Value::Ten, Suit::Club);
    let hand = SelectHand::new(vec![two, three, four, five, ten]);

    // Score flush without joker
    // flush (level 1) -> 35 chips, 4 mult
    // Played cards (2, 3, 4, 5, 10) -> 24 chips
    // (24 + 35) * (4) = 236
    let before = 236;
    // Score flush with joker
    // flush (level 1) -> 35 chips, 4 mult
    // Played cards (2, 3, 4, 5, 10) -> 24 chips
    // joker w/ flush = +10 mult
    // (24 + 35) * (4 + 10) = 826
    let after = 826;

    let j = Jokers::DrollJoker(DrollJoker::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_the_duo_with_pair() {
    let ac = Card::new(Value::Ace, Suit::Club);
    let hand = SelectHand::new(vec![ac, ac, ac, ac]);

    // Score 4ok without joker
    // 4ok (level 1) -> 60 chips, 7 mult
    // Played cards (4 ace) -> 44 chips
    // (60 + 44) * (7) = 728
    let before = 728;
    // Score 4ok with joker (contains a pair)
    // joker w/ pair = X2 mult
    // (60 + 44) * (7 * 2) = 1456
    let after = 1456;

    let j = Jokers::TheDuo(TheDuo::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_the_duo_no_pair() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);

    let before = 16;
    let after = 16;

    let j = Jokers::TheDuo(TheDuo::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_the_trio_with_three_of_kind() {
    let ac = Card::new(Value::Ace, Suit::Club);
    let hand = SelectHand::new(vec![ac, ac, ac, ac]);

    let before = 728;
    // joker w/ three of a kind = X3 mult
    // (60 + 44) * (7 * 3) = 2184
    let after = 2184;

    let j = Jokers::TheTrio(TheTrio::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_the_trio_no_three_of_kind() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);

    let before = 16;
    let after = 16;

    let j = Jokers::TheTrio(TheTrio::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_the_family_with_four_of_kind() {
    let ac = Card::new(Value::Ace, Suit::Club);
    let hand = SelectHand::new(vec![ac, ac, ac, ac]);

    let before = 728;
    // joker w/ four of a kind = X4 mult
    // (60 + 44) * (7 * 4) = 2912
    let after = 2912;

    let j = Jokers::TheFamily(TheFamily::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_the_family_no_four_of_kind() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);

    let before = 16;
    let after = 16;

    let j = Jokers::TheFamily(TheFamily::default());
    score_before_after_joker(j, hand, before, after);
}

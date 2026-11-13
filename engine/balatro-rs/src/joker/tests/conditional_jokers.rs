use super::*;

#[test]
fn test_devious_joker() {
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
    // joker w/ straight = +100 chips
    // (20 + 30 + 100) * (4) = 600
    let after = 600;

    let j = Jokers::DeviousJoker(DeviousJoker::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_crafty_joker() {
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
    // joker w/ flush = +80 chips
    // (24 + 35 + 80) * (4) = 556
    let after = 556;
    let j = Jokers::CraftyJoker(CraftyJoker::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_half_joker() {
    let ac = Card::new(Value::Ace, Suit::Club);
    let hand = SelectHand::new(vec![ac, ac, ac]);

    // Score 3ok without joker
    // 3ok (level 1) -> 30 chips, 3 mult
    // Played cards (3 ace) -> 33 chips
    // (30 + 33) * 3 = 189
    let before = 189;
    // Score 3ok with joker
    // 3ok (level 1) -> 30 chips, 3 mult
    // Played cards (3 ace) -> 33 chips
    // joker w/ <=3 cards = +20 mult
    // (30 + 33) * (3 + 20) = 1449
    let after = 1449;

    let j = Jokers::HalfJoker(HalfJoker::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_joker_stencil() {
    let ace = Card::new(Value::Ace, Suit::Club);
    let hand = SelectHand::new(vec![ace]);
    let best = hand.best_hand().unwrap();

    // High card (level 1) -> 5 chips, 1 mult
    // Played cards (1 ace) -> 11 chips
    // (5 + 11) * 1 = 16
    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    assert_eq!(g.calc_score(best.clone()), 16);

    // Stencil alone in 5 slots: 4 empty + 1 Stencil -> X5
    // (5 + 11) * (1 * 5) = 80
    let j = Jokers::JokerStencil(JokerStencil::default());
    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    assert_eq!(g.calc_score(best.clone()), 80);

    // Add another joker -> 3 empty -> X3
    let j2 = Jokers::Banner(Banner::default());
    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j2.clone());
    g.buy_joker(j2).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    // 3 empty + 1 Stencil -> X4: (5 + 11 + 4*30) * (1 * 4) = 544
    assert_eq!(g.calc_score(best.clone()), 544);
}

#[test]
fn test_banner_joker() {
    let ace = Card::new(Value::Ace, Suit::Club);
    let hand = SelectHand::new(vec![ace]);
    let best = hand.best_hand().unwrap();
    let j = Jokers::Banner(Banner::default());

    // High card (level 1) -> 5 chips, 1 mult
    // Played cards (1 ace) -> 11 chips
    // (5 + 11) * (1) = 16
    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    assert_eq!(g.calc_score(best.clone()), 16);

    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // Banner: 4 discards * 30 chips = +120
    // (5 + 11 + 120) * 1 = 136
    assert_eq!(g.calc_score(best.clone()), 136);

    g.discards_remaining = 0;
    // Banner: +0 chips
    // (5 + 11 + 0) * 1 = 16
    assert_eq!(g.calc_score(best.clone()), 16);
}

#[test]
fn test_mystic_summit() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);
    let best = hand.best_hand().unwrap();
    let j = Jokers::MysticSummit(MysticSummit::default());

    // High card (level 1): 5 chips, 1 mult
    // Played (1 ace): 11 chips
    // (5 + 11) * 1 = 16
    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    assert_eq!(g.calc_score(best.clone()), 16);

    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // discards = 4 (default), so Mystic Summit does NOT fire
    // (5 + 11) * 1 = 16
    assert_eq!(g.calc_score(best.clone()), 16);

    // Now set discards to 0 -> +15 mult
    g.discards_remaining = 0;
    // (5 + 11) * (1 + 15) = 16 * 16 = 256
    assert_eq!(g.calc_score(best.clone()), 256);
}

#[test]
fn test_fibonacci() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let two = Card::new(Value::Two, Suit::Heart);
    let three = Card::new(Value::Three, Suit::Heart);
    let five = Card::new(Value::Five, Suit::Heart);
    let eight = Card::new(Value::Eight, Suit::Heart);
    let hand = SelectHand::new(vec![ace, two, three, five, eight]);
    let j = Jokers::Fibonacci(Fibonacci::default());

    // Flush (level 1): 35 chips, 4 mult
    // Played (5 cards): 11 + 2 + 3 + 5 + 8 = 29 chips
    // (35 + 29) * 4 = 256
    let before = 256;

    // Fibonacci: 1 ace, 1 two, 1 three, 1 five, 1 eight -> +8 mult each
    // (35 + 29) * (4 + 40) = 64 * 44 = 2816
    let after = 2816;
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_scary_face() {
    let ace = Card::new(Value::Ace, Suit::Club);
    let king = Card::new(Value::King, Suit::Club);
    let queen = Card::new(Value::Queen, Suit::Spade);
    let jack = Card::new(Value::Jack, Suit::Heart);
    let ten = Card::new(Value::Ten, Suit::Diamond);
    let hand = SelectHand::new(vec![ace, jack, queen, king, ten]);
    let j = Jokers::ScaryFace(ScaryFace::default());

    // Straight (level 1): 30 chips, 4 mult
    // Played (5 cards): 11 + 10 + 10 + 10 + 10 = 51 chips
    // (30 + 51) * 4 = 324
    let before = 324;

    // Scary Face: jack, queen, king -> +30 chips each
    // (30 + 51 + 90) * 4 = 684
    let after = 684;
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_abstract_joker() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);
    let best = hand.best_hand().unwrap();

    // High card (level 1): 5 chips, 1 mult
    // Played (1 ace): 11 chips
    // (5 + 11) * 1 = 16
    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    assert_eq!(g.calc_score(best.clone()), 16);

    // Buy Abstract Joker -> 1 joker, +3 mult
    // (5 + 11) * (1 + 3) = 64
    g.money += 1000;
    g.stage = Stage::Shop();
    let aj = Jokers::AbstractJoker(AbstractJoker::default());
    g.shop.jokers.push(aj.clone());
    g.buy_joker(aj).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    assert_eq!(g.calc_score(best.clone()), 64);

    // Buy Scary Face -> 2 jokers, +6 mult
    // (5 + 11) * (1 + 6) = 112
    g.money += 1000;
    g.stage = Stage::Shop();
    let sf = Jokers::ScaryFace(ScaryFace::default());
    g.shop.jokers.push(sf.clone());
    g.buy_joker(sf).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    assert_eq!(g.calc_score(best.clone()), 112);
}

#[test]
fn test_pareidolia_scary_face() {
    // Ace low straight: no natural face cards
    let ace = Card::new(Value::Ace, Suit::Club);
    let two = Card::new(Value::Two, Suit::Heart);
    let three = Card::new(Value::Three, Suit::Spade);
    let four = Card::new(Value::Four, Suit::Diamond);
    let five = Card::new(Value::Five, Suit::Club);
    let hand = SelectHand::new(vec![ace, two, three, four, five]);
    let best = hand.best_hand().unwrap();

    // Straight (level 1): 30 chips, 4 mult
    // Card chips: 11 + 2 + 3 + 4 + 5 = 25
    // (30 + 25) * 4 = 220
    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    assert_eq!(g.calc_score(best.clone()), 220);

    // Add Scary Face: still no face cards, so still 220
    g.money += 1000;
    g.stage = Stage::Shop();
    let sf = Jokers::ScaryFace(ScaryFace::default());
    g.shop.jokers.push(sf.clone());
    g.buy_joker(sf).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    assert_eq!(g.calc_score(best.clone()), 220);

    // Add Pareidolia: now all cards are face cards
    // Scary Face gives +30 chips × 5 = +150
    // (30 + 25 + 150) * 4 = 820
    g.money += 1000;
    g.stage = Stage::Shop();
    let p = Jokers::Pareidolia(Pareidolia::default());
    g.shop.jokers.push(p.clone());
    g.buy_joker(p).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    assert_eq!(g.calc_score(best.clone()), 820);
}

#[test]
fn test_even_steven() {
    let two = Card::new(Value::Two, Suit::Club);
    let four = Card::new(Value::Four, Suit::Club);
    let six = Card::new(Value::Six, Suit::Club);
    let eight = Card::new(Value::Eight, Suit::Club);
    let ten = Card::new(Value::Ten, Suit::Club);
    let hand = SelectHand::new(vec![two, four, six, eight, ten]);
    let j = Jokers::EvenSteven(EvenSteven::default());

    // Flush (level 1): 35 chips, 4 mult
    // Played (5 cards): 2 + 4 + 6 + 8 + 10 = 30 chips
    // (35 + 30) * 4 = 260
    let before = 260;

    // Even Steven: 5 even cards * +4 mult = +20 mult
    // (35 + 30) * (4 + 20) = 65 * 24 = 1560
    let after = 1560;
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_even_steven_odd_cards() {
    let three = Card::new(Value::Three, Suit::Club);
    let five = Card::new(Value::Five, Suit::Club);
    let seven = Card::new(Value::Seven, Suit::Club);
    let nine = Card::new(Value::Nine, Suit::Club);
    let ace = Card::new(Value::Ace, Suit::Club);
    let hand = SelectHand::new(vec![three, five, seven, nine, ace]);
    let j = Jokers::EvenSteven(EvenSteven::default());

    // Flush (level 1): 35 chips, 4 mult
    // Played (5 cards): 3 + 5 + 7 + 9 + 11 = 35 chips
    // (35 + 35) * 4 = 280
    let before = 280;

    // Even Steven: 0 even cards -> no bonus
    let after = 280;
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_odd_todd() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let three = Card::new(Value::Three, Suit::Heart);
    let five = Card::new(Value::Five, Suit::Heart);
    let seven = Card::new(Value::Seven, Suit::Heart);
    let nine = Card::new(Value::Nine, Suit::Heart);
    let hand = SelectHand::new(vec![ace, three, five, seven, nine]);
    let j = Jokers::OddTodd(OddTodd::default());

    // Flush (level 1): 35 chips, 4 mult
    // Played (5 cards): 11 + 3 + 5 + 7 + 9 = 35 chips
    // (35 + 35) * 4 = 280
    let before = 280;

    // Odd Todd: 5 odd cards * +31 chips = +155 chips
    // (35 + 35 + 155) * 4 = 225 * 4 = 900
    let after = 900;
    score_before_after_joker(j, hand, before, after);
}

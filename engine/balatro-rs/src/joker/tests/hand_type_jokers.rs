use super::*;

#[test]
fn test_the_order_with_straight() {
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
    // joker w/ straight = X3 mult
    // (20 + 30) * (4 * 3) = 600
    let after = 600;

    let j = Jokers::TheOrder(TheOrder::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_the_order_no_straight() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);

    let before = 16;
    let after = 16;

    let j = Jokers::TheOrder(TheOrder::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_the_tribe_with_flush() {
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
    // joker w/ flush = X2 mult
    // (24 + 35) * (4 * 2) = 472
    let after = 472;

    let j = Jokers::TheTribe(TheTribe::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_the_tribe_no_flush() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);

    let before = 16;
    let after = 16;

    let j = Jokers::TheTribe(TheTribe::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_stone_joker_counts_deck_and_discarded() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);
    let j = Jokers::StoneJoker(StoneJoker::default());

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    let mut deck_stone = Card::new(Value::Two, Suit::Diamond);
    deck_stone.enhancement = Some(Enhancement::Stone);
    g.deck.push(deck_stone);

    let mut discarded_stone = Card::new(Value::Three, Suit::Diamond);
    discarded_stone.enhancement = Some(Enhancement::Stone);
    g.discarded.push(discarded_stone);

    assert_eq!(g.calc_score(hand.best_hand().unwrap()), 16);

    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // 2 Stone cards total (1 undrawn in deck, 1 already discarded) -> +50 chips
    // (5 + 11 + 50) * 1 = 66
    assert_eq!(g.calc_score(hand.best_hand().unwrap()), 66);
}

#[test]
fn test_stone_joker_no_stone_cards() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);

    let before = 16;
    let after = 16;

    let j = Jokers::StoneJoker(StoneJoker::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_steel_joker_counts_deck_and_discarded() {
    let ac = Card::new(Value::Ace, Suit::Club);
    let hand = SelectHand::new(vec![ac, ac, ac, ac]);
    let j = Jokers::SteelJoker(SteelJoker::default());

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    let mut deck_steel = Card::new(Value::Two, Suit::Diamond);
    deck_steel.enhancement = Some(Enhancement::Steel);
    g.deck.push(deck_steel);

    let mut discarded_steel = Card::new(Value::Three, Suit::Diamond);
    discarded_steel.enhancement = Some(Enhancement::Steel);
    g.discarded.push(discarded_steel);

    // Score 4ok without joker
    // 4ok (level 1) -> 60 chips, 7 mult
    // Played cards (4 ace) -> 44 chips
    // (60 + 44) * 7 = 728
    assert_eq!(g.calc_score(hand.best_hand().unwrap()), 728);

    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // 2 Steel cards total -> X1.4: 7 * 1.4 = 9.8
    // floor((60 + 44) * 9.8) = 1019
    assert_eq!(g.calc_score(hand.best_hand().unwrap()), 1019);
}

#[test]
fn test_steel_joker_no_steel_cards() {
    let ac = Card::new(Value::Ace, Suit::Club);
    let hand = SelectHand::new(vec![ac, ac, ac, ac]);

    let before = 728;
    let after = 728;

    let j = Jokers::SteelJoker(SteelJoker::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_blue_joker_excludes_drawn_cards() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);
    let j = Jokers::BlueJoker(BlueJoker::default());

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    // simulate 10 cards already drawn out of the deck this round
    let drawn = g.deck.draw(10);
    g.available.extend(drawn);
    assert_eq!(g.deck.len(), 42);

    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // BlueJoker only counts the undrawn remainder (42), not the 10 held cards
    // High card (level 1): 5 chips, 1 mult
    // Played (1 ace): 11 chips
    // BlueJoker: +2 chips per remaining deck card = 42 * 2 = 84
    // (5 + 11 + 84) * 1 = 100
    assert_eq!(g.calc_score(hand.best_hand().unwrap()), 100);
}

#[test]
fn test_blue_joker_fresh_deck() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);

    // Full 52-card deck untouched -> +2*52 = 104 chips
    // (5 + 11 + 104) * 1 = 120
    let before = 16;
    let after = 120;

    let j = Jokers::BlueJoker(BlueJoker::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_erosion_with_deficit() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);
    let j = Jokers::Erosion(Erosion::default());

    let mut g = Game::default();
    // draw 10 cards into hand -- still owned this run, not a deficit
    let drawn = g.deck.draw(10);
    g.available.extend(drawn);
    // destroy exactly 1 card -- a genuine deficit of 1 below the 52-card starting size
    let victim = g.deck.cards()[0];
    g.destroy_card(victim.id);
    assert_eq!(g.full_deck().len(), 51);

    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // The 10 drawn cards are still owned (no deficit from those); only the
    // destroyed card counts -> deficit of 1 -> +4 mult
    // (5 + 11) * (1 + 4) = 80
    assert_eq!(g.calc_score(hand.best_hand().unwrap()), 80);
}

#[test]
fn test_erosion_no_deficit() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);

    let before = 16;
    let after = 16;

    let j = Jokers::Erosion(Erosion::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_drivers_license_with_enough_enhanced_cards() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);
    let j = Jokers::DriversLicense(DriversLicense::default());

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };

    // enhance 10 cards already in the deck
    let ids: Vec<usize> = g.deck.cards().iter().take(10).map(|c| c.id).collect();
    for id in ids {
        g.mutate_card(id, |c| c.enhancement = Some(Enhancement::Bonus));
    }

    // 6 more enhanced cards already discarded this round (not in the deck at all)
    for _ in 0..6 {
        let mut c = Card::new(Value::Two, Suit::Diamond);
        c.enhancement = Some(Enhancement::Bonus);
        g.discarded.push(c);
    }

    assert_eq!(
        g.full_deck()
            .iter()
            .filter(|c| c.enhancement.is_some())
            .count(),
        16
    );

    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // 16 enhanced cards across deck+discarded (>= 16 threshold) -> X3 mult
    // (5 + 11) * (1 * 3) = 48
    assert_eq!(g.calc_score(hand.best_hand().unwrap()), 48);
}

#[test]
fn test_drivers_license_not_enough_enhanced_cards() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);

    let before = 16;
    let after = 16;

    let j = Jokers::DriversLicense(DriversLicense::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_sly_joker() {
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
    // joker w/ pair = +50 chips
    // (60 + 44 + 50) * (7) = 1078
    let after = 1078;

    let j = Jokers::SlyJoker(SlyJoker::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_wily_joker() {
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
    // joker w/ 3ok = +100 chips
    // (60 + 44 + 100) * (7) = 1428
    let after = 1428;

    let j = Jokers::WilyJoker(WilyJoker::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_clever_joker() {
    let ac = Card::new(Value::Ace, Suit::Club);
    let kc = Card::new(Value::King, Suit::Club);
    let hand = SelectHand::new(vec![ac, ac, kc, kc]);

    // Score two pair without joker
    // two pair (level 1) -> 20 chips, 2 mult
    // Played cards (2 ace, 2 king) -> 42 chips
    // (20 + 42) * (2) = 124
    let before = 124;
    // Score two pair with joker
    // two pair (level 1) -> 20 chips, 2 mult
    // Played cards (2 ace, 2 king) -> 42 chips
    // joker w/ two pair = +80 chips
    // (20 + 42 + 80) * (2) = 284
    let after = 284;

    let j = Jokers::CleverJoker(CleverJoker::default());
    score_before_after_joker(j, hand, before, after);
}

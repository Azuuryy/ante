use super::*;

#[test]
fn test_midas_mask_does_not_convert_unscored_kicker() {
    let king1 = Card::new(Value::King, Suit::Heart);
    let king2 = Card::new(Value::King, Suit::Spade);
    let queen = Card::new(Value::Queen, Suit::Diamond);

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.available.extend(vec![king1, king2, queen]);

    g.money += 1000;
    g.stage = Stage::Shop();
    let j = Jokers::MidasMask(MidasMask::default());
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // Pair of Kings + Queen kicker: only the pair is scored, the queen
    // kicker never scores and so should stay unenhanced
    let hand = SelectHand::new(vec![king1, king2, queen]);
    g.calc_score(hand.best_hand().unwrap());

    let cards = g.available.cards();
    let scored_king = cards.iter().find(|c| c.id == king1.id).unwrap();
    let kicker_queen = cards.iter().find(|c| c.id == queen.id).unwrap();
    assert_eq!(scored_king.enhancement, Some(Enhancement::Gold));
    assert_eq!(kicker_queen.enhancement, None);
}

#[test]
fn test_photograph() {
    let king = Card::new(Value::King, Suit::Heart);
    let hand = SelectHand::new(vec![king]);
    let j = Jokers::Photograph(Photograph::default());

    // High card (level 1): 5 chips, 1 mult
    // Played (1 king): 10 chips
    // (5 + 10) * 1 = 15
    let before = 15;

    // Photograph: 1 face card -> X2 mult
    // (5 + 10) * (1 * 2) = 30
    let after = 30;
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_photograph_no_face() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);
    let j = Jokers::Photograph(Photograph::default());

    // High card (level 1): 5 chips, 1 mult
    // Played (1 ace): 11 chips
    // (5 + 11) * 1 = 16
    let before = 16;

    // Photograph: 0 face cards -> no bonus
    let after = 16;
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_reserved_parking() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);
    let j = Jokers::ReservedParking(ReservedParking::default());

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.available.extend(vec![
        Card::new(Value::King, Suit::Club),
        Card::new(Value::Queen, Suit::Spade),
        Card::new(Value::Jack, Suit::Heart),
    ]);
    let best = hand.best_hand().unwrap();
    g.calc_score(best.clone());

    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    let mut saw_increase = false;
    for _ in 0..100 {
        g.money = 994;
        g.calc_score(best.clone());
        if g.money > 994 {
            saw_increase = true;
            break;
        }
    }
    assert!(saw_increase, "Reserved Parking should sometimes give money");
}

#[test]
fn test_reserved_parking_no_face() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);
    let j = Jokers::ReservedParking(ReservedParking::default());

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.available.extend(vec![
        Card::new(Value::Ace, Suit::Club),
        Card::new(Value::Two, Suit::Spade),
    ]);
    let best = hand.best_hand().unwrap();
    g.calc_score(best.clone());

    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    let money_before = g.money;
    g.calc_score(best.clone());
    assert_eq!(g.money, money_before);
}

#[test]
fn test_reserved_parking_pareidolia_counts_non_face_cards() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);
    let rp = Jokers::ReservedParking(ReservedParking::default());
    let p = Jokers::Pareidolia(Pareidolia::default());

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.available.extend(vec![
        Card::new(Value::Ace, Suit::Club),
        Card::new(Value::Two, Suit::Spade),
    ]);
    let best = hand.best_hand().unwrap();
    g.calc_score(best.clone());

    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(rp.clone());
    g.buy_joker(rp).unwrap();
    g.shop.jokers.push(p.clone());
    g.buy_joker(p).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    let mut saw_increase = false;
    for _ in 0..100 {
        g.money = 994;
        g.calc_score(best.clone());
        if g.money > 994 {
            saw_increase = true;
            break;
        }
    }
    assert!(
        saw_increase,
        "Pareidolia should make non-face held cards count for Reserved Parking"
    );
}

#[test]
fn test_baseball_card() {
    let ten = Card::new(Value::Ten, Suit::Heart);
    let hand = SelectHand::new(vec![ten, ten]);
    let best = hand.best_hand().unwrap();

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };

    // Pair (level 1): 10 chips, 2 mult
    // Played (2 tens): 20 chips
    // (10 + 20) * 2 = 60
    assert_eq!(g.calc_score(best.clone()), 60);

    // Buy 2 uncommon jokers (MidasMask, Pareidolia) and BaseballCard
    let midas = Jokers::MidasMask(MidasMask::default());
    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(midas.clone());
    g.buy_joker(midas).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    g.calc_score(best.clone());

    let pareidolia = Jokers::Pareidolia(Pareidolia::default());
    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(pareidolia.clone());
    g.buy_joker(pareidolia).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    g.calc_score(best.clone());

    let bb = Jokers::BaseballCard(BaseballCard::default());
    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(bb.clone());
    g.buy_joker(bb).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // BaseballCard: 2 uncommon * X1.5
    // (10 + 20) * (2 * 1.5 * 1.5) = 30 * 4.5 = 135
    assert_eq!(g.calc_score(best.clone()), 135);
}

#[test]
fn test_baseball_card_no_uncommon() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);
    let best = hand.best_hand().unwrap();

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };

    // Buy BaseballCard with no uncommon jokers
    let bb = Jokers::BaseballCard(BaseballCard::default());
    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(bb.clone());
    g.buy_joker(bb).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // High card (level 1): 5 chips, 1 mult
    // Played (1 ace): 11 chips
    // (5 + 11) * 1 = 16
    assert_eq!(g.calc_score(best.clone()), 16);
}

#[test]
fn test_bull() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);
    let j = Jokers::Bull(Bull::default());

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        money: 100,
        ..Default::default()
    };
    let best = hand.best_hand().unwrap();

    // High card (level 1): 5 chips, 1 mult
    // Played (1 ace): 11 chips
    // (5 + 11) * 1 = 16
    assert_eq!(g.calc_score(best.clone()), 16);

    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // Bull: (100 + 1000 - 6) * 2 = 1094 * 2 = 2188 chips
    // (5 + 11 + 2188) * 1 = 2204
    assert_eq!(g.calc_score(best.clone()), 2204);
}

#[test]
fn test_bull_no_money() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);
    let j = Jokers::Bull(Bull::default());

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    let best = hand.best_hand().unwrap();

    // High card (level 1): 5 chips, 1 mult
    // Played (1 ace): 11 chips
    // (5 + 11) * 1 = 16
    assert_eq!(g.calc_score(best.clone()), 16);

    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    g.money = 0;

    // Bull: 0 * 2 = 0 chips
    // (5 + 11) * 1 = 16
    assert_eq!(g.calc_score(best.clone()), 16);
}

#[test]
fn test_walkie_talkie() {
    let ten = Card::new(Value::Ten, Suit::Heart);
    let hand = SelectHand::new(vec![ten, ten]);
    let j = Jokers::WalkieTalkie(WalkieTalkie::default());

    // Pair (level 1): 10 chips, 2 mult
    // Played (2 tens): 20 chips
    // (10 + 20) * 2 = 60
    let before = 60;

    // WalkieTalkie: 2 tens * (+10 chips, +4 mult) = +20 chips, +8 mult
    // (10 + 20 + 20) * (2 + 8) = 50 * 10 = 500
    let after = 500;
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_walkie_talkie_other_cards() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);
    let j = Jokers::WalkieTalkie(WalkieTalkie::default());

    // High card (level 1): 5 chips, 1 mult
    // Played (1 ace): 11 chips
    // (5 + 11) * 1 = 16
    let before = 16;

    // WalkieTalkie: 0 tens or fours -> no bonus
    let after = 16;
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_smiley_face() {
    let king = Card::new(Value::King, Suit::Heart);
    let king2 = Card::new(Value::King, Suit::Diamond);
    let hand = SelectHand::new(vec![king, king2]);
    let j = Jokers::SmileyFace(SmileyFace::default());

    // Pair (level 1): 10 chips, 2 mult
    // Played (2 kings): 10 + 10 = 20 chips
    // (10 + 20) * 2 = 60
    let before = 60;

    // Smiley Face: 2 face cards * +5 mult = +10 mult
    // (10 + 20) * (2 + 10) = 30 * 12 = 360
    let after = 360;
    score_before_after_joker(j, hand, before, after);
}

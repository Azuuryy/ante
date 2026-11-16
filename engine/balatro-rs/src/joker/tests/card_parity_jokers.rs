use super::*;

#[test]
fn test_odd_todd_even_cards() {
    let two = Card::new(Value::Two, Suit::Club);
    let four = Card::new(Value::Four, Suit::Club);
    let six = Card::new(Value::Six, Suit::Club);
    let eight = Card::new(Value::Eight, Suit::Club);
    let ten = Card::new(Value::Ten, Suit::Club);
    let hand = SelectHand::new(vec![two, four, six, eight, ten]);
    let j = Jokers::OddTodd(OddTodd::default());

    // Flush (level 1): 35 chips, 4 mult
    // Played (5 cards): 2 + 4 + 6 + 8 + 10 = 30 chips
    // (35 + 30) * 4 = 260
    let before = 260;

    // Odd Todd: 0 odd cards -> no bonus
    let after = 260;
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_even_steven_pareidolia_disqualifies_even_cards() {
    // High Card Two, alone: (5 + 2) * 1 = 7
    // + EvenSteven: Two is even -> +4 mult: (5 + 2) * (1 + 4) = 35
    // + Pareidolia: Two now counts as a face card, so no longer "even": back to 7
    let two = Card::new(Value::Two, Suit::Heart);
    let hand = SelectHand::new(vec![two]);

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    let best = hand.best_hand().unwrap();
    assert_eq!(g.calc_score(best.clone()), 7);

    g.money += 1000;
    g.stage = Stage::Shop();
    let es = Jokers::EvenSteven(EvenSteven::default());
    g.shop.jokers.push(es.clone());
    g.buy_joker(es).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    assert_eq!(g.calc_score(best.clone()), 35);

    g.money += 1000;
    g.stage = Stage::Shop();
    let p = Jokers::Pareidolia(Pareidolia::default());
    g.shop.jokers.push(p.clone());
    g.buy_joker(p).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    assert_eq!(g.calc_score(best.clone()), 7);
}

#[test]
fn test_odd_todd_pareidolia_disqualifies_odd_cards() {
    // High Card Three, alone: (5 + 3) * 1 = 8
    // + OddTodd: Three is odd -> +31 chips: (5 + 3 + 31) * 1 = 39
    // + Pareidolia: Three now counts as a face card, so no longer "odd": back to 8
    let three = Card::new(Value::Three, Suit::Heart);
    let hand = SelectHand::new(vec![three]);

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    let best = hand.best_hand().unwrap();
    assert_eq!(g.calc_score(best.clone()), 8);

    g.money += 1000;
    g.stage = Stage::Shop();
    let ot = Jokers::OddTodd(OddTodd::default());
    g.shop.jokers.push(ot.clone());
    g.buy_joker(ot).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    assert_eq!(g.calc_score(best.clone()), 39);

    g.money += 1000;
    g.stage = Stage::Shop();
    let p = Jokers::Pareidolia(Pareidolia::default());
    g.shop.jokers.push(p.clone());
    g.buy_joker(p).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    assert_eq!(g.calc_score(best.clone()), 8);
}

#[test]
fn test_scholar() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace, ace]);
    let j = Jokers::Scholar(Scholar::default());

    // Pair (level 1): 10 chips, 2 mult
    // Played (2 aces): 2 * 11 = 22 chips
    // (10 + 22) * 2 = 64
    let before = 64;

    // Scholar: 2 aces * (+20 chips, +4 mult) = +40 chips, +8 mult
    // (10 + 22 + 40) * (2 + 8) = 72 * 10 = 720
    let after = 720;
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_scholar_no_aces() {
    let king = Card::new(Value::King, Suit::Club);
    let hand = SelectHand::new(vec![king]);
    let j = Jokers::Scholar(Scholar::default());

    // High card (level 1): 5 chips, 1 mult
    // Played (1 king): 10 chips
    // (5 + 10) * 1 = 15
    let before = 15;

    // Scholar: 0 aces -> no bonus
    let after = 15;
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_business_card_no_face_cards() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace, ace]);
    let j = Jokers::BusinessCard(BusinessCard::default());

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    let score = g.calc_score(hand.best_hand().unwrap());
    assert_eq!(score, 64);

    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    let money_before = g.money;
    g.calc_score(hand.best_hand().unwrap());
    assert_eq!(g.money, money_before);
}

#[test]
fn test_business_card_face_cards() {
    let king = Card::new(Value::King, Suit::Heart);
    let queen = Card::new(Value::Queen, Suit::Heart);
    let jack = Card::new(Value::Jack, Suit::Heart);
    let hand = SelectHand::new(vec![king, queen, jack]);
    let j = Jokers::BusinessCard(BusinessCard::default());

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.calc_score(hand.best_hand().unwrap());

    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    let mut saw_increase = false;
    for _ in 0..100 {
        g.money = 1000;
        g.calc_score(hand.best_hand().unwrap());
        if g.money > 1000 {
            saw_increase = true;
            break;
        }
    }
    assert!(saw_increase, "Business Card should sometimes give money");
}

#[test]
fn test_faceless_joker() {
    let j = Jokers::FacelessJoker(FacelessJoker::default());

    let mut g = Game::default();
    g.start();
    g.stage = Stage::Blind(Blind::Small);
    g.blind = Some(Blind::Small);
    g.deal();

    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // At least 3 face cards among the discarded ones.
    let cards: Vec<Card> = g.available.cards().iter().take(5).copied().collect();
    for card in &cards[..3] {
        g.mutate_card(card.id, |c| c.value = Value::King);
    }
    for card in &cards[3..] {
        g.mutate_card(card.id, |c| c.value = Value::Two);
    }
    for card in g.available.cards().iter().take(5) {
        g.available.select_card(*card).expect("can select");
    }
    assert_eq!(g.available.selected().len(), 5);

    g.discard_selected().expect("can discard");
    assert_eq!(g.money, 1001);
}

#[test]
fn test_faceless_joker_few_cards() {
    let j = Jokers::FacelessJoker(FacelessJoker::default());

    let mut g = Game::default();
    g.start();
    g.stage = Stage::Blind(Blind::Small);
    g.blind = Some(Blind::Small);
    g.deal();

    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // Three cards that are not faces.
    let cards: Vec<Card> = g.available.cards().iter().take(3).copied().collect();
    for card in &cards {
        g.mutate_card(card.id, |c| c.value = Value::Two);
    }
    for card in g.available.cards().iter().take(3) {
        g.available.select_card(*card).expect("can select");
    }
    assert_eq!(g.available.selected().len(), 3);

    g.discard_selected().expect("can discard");
    assert_eq!(g.money, 996);
}

#[test]
fn test_baron() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace, ace]);
    let j = Jokers::Baron(Baron::default());

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.available.extend(vec![
        Card::new(Value::King, Suit::Club),
        Card::new(Value::King, Suit::Spade),
    ]);
    let best = hand.best_hand().unwrap();

    // Pair (level 1): 10 chips, 2 mult
    // Played (2 aces): 22 chips
    // (10 + 22) * 2 = 64
    assert_eq!(g.calc_score(best.clone()), 64);

    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // Baron: 2 kings -> 2 * 1.5 * 1.5 = 4.5
    // (10 + 22) * 4.5 = 144
    assert_eq!(g.calc_score(best.clone()), 144);
}

#[test]
fn test_baron_no_kings() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace, ace]);
    let j = Jokers::Baron(Baron::default());

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.available.extend(vec![
        Card::new(Value::Queen, Suit::Club),
        Card::new(Value::Jack, Suit::Spade),
    ]);
    let best = hand.best_hand().unwrap();

    // Pair (level 1): 10 chips, 2 mult
    // Played (2 aces): 22 chips
    // (10 + 22) * 2 = 64
    assert_eq!(g.calc_score(best.clone()), 64);

    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // Baron: 0 kings -> no bonus
    assert_eq!(g.calc_score(best.clone()), 64);
}

#[test]
fn test_midas_mask() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace, ace]);
    let j = Jokers::MidasMask(MidasMask::default());

    // Pair (level 1): 10 chips, 2 mult
    // Played (2 aces): 22 chips
    // (10 + 22) * 2 = 64
    let before = 64;

    // Aces aren't face cards, so Midas Mask doesn't touch them -> same score
    let after = 64;
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_midas_mask_converts_played_face_card_to_gold() {
    let king = Card::new(Value::King, Suit::Heart);

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.available.extend(vec![king]);

    g.money += 1000;
    g.stage = Stage::Shop();
    let j = Jokers::MidasMask(MidasMask::default());
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    let hand = SelectHand::new(vec![king]);
    g.calc_score(hand.best_hand().unwrap());

    // the enhancement must persist onto the real card, not just the
    // transient MadeHand used for this scoring pass
    let scored = g.available.cards().into_iter().find(|c| c.id == king.id);
    assert_eq!(scored.unwrap().enhancement, Some(Enhancement::Gold));
}

#[test]
fn test_midas_mask_pareidolia_converts_non_face_card_to_gold() {
    let two = Card::new(Value::Two, Suit::Heart);

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.available.extend(vec![two]);

    g.money += 1000;
    g.stage = Stage::Shop();
    let mm = Jokers::MidasMask(MidasMask::default());
    g.shop.jokers.push(mm.clone());
    g.buy_joker(mm).unwrap();
    let p = Jokers::Pareidolia(Pareidolia::default());
    g.shop.jokers.push(p.clone());
    g.buy_joker(p).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    let hand = SelectHand::new(vec![two]);
    g.calc_score(hand.best_hand().unwrap());

    let scored = g.available.cards().into_iter().find(|c| c.id == two.id);
    assert_eq!(scored.unwrap().enhancement, Some(Enhancement::Gold));
}

use super::*;

#[test]
fn test_smiley_face_no_face() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);
    let j = Jokers::SmileyFace(SmileyFace::default());

    // High card (level 1): 5 chips, 1 mult
    // Played (1 ace): 11 chips
    // (5 + 11) * 1 = 16
    let before = 16;

    // Smiley Face: 0 face cards -> no bonus
    let after = 16;
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_golden_ticket_no_gold() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace, ace]);
    let j = Jokers::GoldenTicket(GoldenTicket::default());

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
fn test_golden_ticket_with_gold() {
    let mut ace = Card::new(Value::Ace, Suit::Heart);
    ace.enhancement = Some(Enhancement::Gold);
    let hand = SelectHand::new(vec![ace]);
    let j = Jokers::GoldenTicket(GoldenTicket::default());

    let mut g = Game::default();
    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    let money_before = g.money;
    g.calc_score(hand.best_hand().unwrap());
    assert_eq!(g.money, money_before + 4);
}

#[test]
fn test_acrobat_final_hand() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);
    let j = Jokers::Acrobat(Acrobat::default());

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

    // Final hand: plays == 0
    g.plays = 0;
    // High card ace: (5 + 11) * 1 = 16
    // Acrobat X3: 16 * 3 = 48
    assert_eq!(g.calc_score(hand.best_hand().unwrap()), 48);
}

#[test]
fn test_acrobat_not_final_hand() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);
    let j = Jokers::Acrobat(Acrobat::default());

    let mut g = Game::default();
    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // Not final hand: plays > 0
    g.plays = 1;
    // High card ace: (5 + 11) * 1 = 16
    // Acrobat: no bonus
    assert_eq!(g.calc_score(hand.best_hand().unwrap()), 16);
}

#[test]
fn test_rough_gem_no_diamonds() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace, ace]);
    let j = Jokers::RoughGem(RoughGem::default());

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

    let money_before = g.money;
    g.calc_score(hand.best_hand().unwrap());
    assert_eq!(g.money, money_before);
}

#[test]
fn test_rough_gem_with_diamonds() {
    let dia1 = Card::new(Value::Ace, Suit::Diamond);
    let dia2 = Card::new(Value::Ace, Suit::Club);
    let hand = SelectHand::new(vec![dia1, dia2]);
    let j = Jokers::RoughGem(RoughGem::default());

    let mut g = Game::default();
    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    let money_before = g.money;
    g.calc_score(hand.best_hand().unwrap());
    assert_eq!(g.money, money_before + 1);
}

#[test]
fn test_bloodstone_no_hearts() {
    let ace = Card::new(Value::Ace, Suit::Club);
    let hand = SelectHand::new(vec![ace]);
    let j = Jokers::Bloodstone(Bloodstone::default());

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

    // High card club: (5 + 11) * 1 = 16, no hearts -> no Xmult
    assert_eq!(g.calc_score(hand.best_hand().unwrap()), 16);
}

#[test]
fn test_bloodstone_with_hearts() {
    let heart = Card::new(Value::Ace, Suit::Heart);
    let heart2 = Card::new(Value::Ace, Suit::Diamond);
    let hand = SelectHand::new(vec![heart, heart2]);
    let j = Jokers::Bloodstone(Bloodstone::default());

    let mut g = Game::default();
    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // Pair of aces (level 1): 10 chips, 2 mult
    // Played: Ace heart (11 chips) + Ace diamond (11 chips) = 22 chips
    // (10 + 22) * 2 = 64
    // With Bloodstone: 1 heart card, 50% chance X1.5
    // Expected: sometimes 64, sometimes 96
    let mut saw_increase = false;
    let mut saw_no_increase = false;
    for _ in 0..50 {
        let score = g.calc_score(hand.best_hand().unwrap());
        if score == 96 {
            saw_increase = true;
        } else if score == 64 {
            saw_no_increase = true;
        }
    }
    assert!(saw_increase, "Bloodstone should sometimes Xmult");
    assert!(saw_no_increase, "Bloodstone should sometimes not Xmult");
}

#[test]
fn test_arrowhead_with_spades() {
    let ace = Card::new(Value::Ace, Suit::Spade);
    let hand = SelectHand::new(vec![ace]);

    // High card (level 1): 5 chips, 1 mult
    // Played (1 ace spade): 11 chips
    // (5 + 11) * 1 = 16
    let before = 16;
    // Arrowhead: +50 chips for the spade
    // (5 + 11 + 50) * 1 = 66
    let after = 66;

    let j = Jokers::Arrowhead(Arrowhead::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_arrowhead_no_spades() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);

    let before = 16;
    let after = 16;

    let j = Jokers::Arrowhead(Arrowhead::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_onyx_agate_with_clubs() {
    let ace = Card::new(Value::Ace, Suit::Club);
    let hand = SelectHand::new(vec![ace]);

    let before = 16;
    // (5 + 11) * (1 + 7) = 128
    let after = 128;

    let j = Jokers::OnyxAgate(OnyxAgate::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_onyx_agate_no_clubs() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);

    let before = 16;
    let after = 16;

    let j = Jokers::OnyxAgate(OnyxAgate::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_shoot_the_moon_with_held_queen() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);
    let j = Jokers::ShootTheMoon(ShootTheMoon::default());

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.available
        .extend(vec![Card::new(Value::Queen, Suit::Spade)]);

    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // High card (level 1): 5 chips, 1 mult
    // Played (1 ace): 11 chips
    // Held Queen: +13 mult
    // (5 + 11) * (1 + 13) = 224
    assert_eq!(g.calc_score(hand.best_hand().unwrap()), 224);
}

#[test]
fn test_shoot_the_moon_no_held_queen() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);
    let j = Jokers::ShootTheMoon(ShootTheMoon::default());

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };

    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // No held cards -> no bonus; (5 + 11) * 1 = 16
    assert_eq!(g.calc_score(hand.best_hand().unwrap()), 16);
}

#[test]
fn test_raised_fist_uses_lowest_held_card() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);
    let j = Jokers::RaisedFist(RaisedFist::default());

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.available.extend(vec![
        Card::new(Value::Two, Suit::Club),
        Card::new(Value::King, Suit::Spade),
    ]);

    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // High card (level 1): 5 chips, 1 mult
    // Played (1 ace): 11 chips
    // Lowest held card: Two (2 chips) -> +4 mult
    // (5 + 11) * (1 + 4) = 80
    assert_eq!(g.calc_score(hand.best_hand().unwrap()), 80);
}

#[test]
fn test_raised_fist_no_held_cards() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);
    let j = Jokers::RaisedFist(RaisedFist::default());

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };

    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // No held cards -> no bonus; (5 + 11) * 1 = 16
    assert_eq!(g.calc_score(hand.best_hand().unwrap()), 16);
}

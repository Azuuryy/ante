use super::*;

#[test]
fn test_obelisk_grows_when_another_hand_is_tied_for_most_played() {
    let high_card_hand = SelectHand::new(vec![Card::new(Value::Ace, Suit::Heart)])
        .best_hand()
        .unwrap();
    let four_aces = Card::new(Value::Ace, Suit::Club);
    let four_of_kind_hand = SelectHand::new(vec![four_aces, four_aces, four_aces, four_aces])
        .best_hand()
        .unwrap();
    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.calc_score(high_card_hand);
    g.money += 1000;
    g.stage = Stage::Shop();
    let j = Jokers::Obelisk(Obelisk::default());
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    // FourOfAKind and HighCard both have 1 play: not strictly the most played
    // floor(104 * 7 * 1.2) = 873
    assert_eq!(g.calc_score(four_of_kind_hand), 873);
}

#[test]
fn test_wee_joker_accumulates_chips_per_scored_two() {
    let two = Card::new(Value::Two, Suit::Heart);
    let two_hand = SelectHand::new(vec![two]).best_hand().unwrap(); // HighCard, one 2

    let ace = Card::new(Value::Ace, Suit::Heart);
    let ace_hand = SelectHand::new(vec![ace]).best_hand().unwrap(); // HighCard, no 2s

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.money += 1000;
    g.stage = Stage::Shop();
    let mut j = Jokers::WeeJoker(WeeJoker::default());
    j.set_instance_id(42);
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // scored 2: counter 0 -> 8; chips = level(5) + card(2) + counter(8) = 15
    assert_eq!(g.calc_score(two_hand.clone()), 15);
    // scored 2 again: counter 8 -> 16; chips = 5 + 2 + 16 = 23
    assert_eq!(g.calc_score(two_hand), 23);
    // no 2 this hand, but the accumulated bonus persists: chips = 5 + 11 + 16 = 32
    assert_eq!(g.calc_score(ace_hand), 32);
}

#[test]
fn test_spare_trousers_accumulates_mult_per_two_pair_hand() {
    let ac = Card::new(Value::Ace, Suit::Club);
    let kc = Card::new(Value::King, Suit::Club);
    let two_pair_hand = SelectHand::new(vec![ac, ac, kc, kc]).best_hand().unwrap(); // TwoPair

    let ace = Card::new(Value::Ace, Suit::Heart);
    let ace_hand = SelectHand::new(vec![ace]).best_hand().unwrap(); // HighCard, no two pair

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.money += 1000;
    g.stage = Stage::Shop();
    let mut j = Jokers::SpareTrousers(SpareTrousers::default());
    j.set_instance_id(99);
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // two pair: counter 0 -> 2; mult = level(2) + counter(2) = 4
    // chips = level(20) + cards(2*11 + 2*10 = 42) = 62; score = 62 * 4 = 248
    assert_eq!(g.calc_score(two_pair_hand.clone()), 248);
    // two pair again: counter 2 -> 4; mult = 2 + 4 = 6; score = 62 * 6 = 372
    assert_eq!(g.calc_score(two_pair_hand), 372);
    // no two pair this hand, bonus persists: mult = level(1) + counter(4) = 5
    // chips = 5 + 11 = 16; score = 16 * 5 = 80
    assert_eq!(g.calc_score(ace_hand), 80);
}

#[test]
fn test_trading_card_destroys_first_single_discard_only() {
    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.money += 1000;
    g.stage = Stage::Shop();
    let mut j = Jokers::TradingCard(TradingCard::default());
    j.set_instance_id(1);
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    let c1 = Card::new(Value::Two, Suit::Heart);
    g.available.extend(vec![c1]);
    g.select_card(c1).unwrap();
    let money_before = g.money;
    g.discard_selected().unwrap();
    // first discard of the round, exactly 1 card: destroyed + $3
    assert_eq!(g.money, money_before + 3);
    assert!(!g.discarded.iter().any(|c| c.id == c1.id));

    let c2 = Card::new(Value::Three, Suit::Club);
    g.available.extend(vec![c2]);
    g.select_card(c2).unwrap();
    let money_before = g.money;
    g.discard_selected().unwrap();
    // second discard of the round, even though it's also 1 card: no-op
    assert_eq!(g.money, money_before);
    assert!(g.discarded.iter().any(|c| c.id == c2.id));
}

#[test]
fn test_green_joker_mult_tracks_hands_played_minus_discards() {
    let ace_hand = SelectHand::new(vec![Card::new(Value::Ace, Suit::Heart)])
        .best_hand()
        .unwrap();

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.money += 1000;
    g.stage = Stage::Shop();
    let mut j = Jokers::GreenJoker(GreenJoker::default());
    j.set_instance_id(1);
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    let discard = |g: &mut Game| {
        let card = Card::new(Value::Two, Suit::Club);
        g.available.extend(vec![card]);
        g.select_card(card).unwrap();
        g.discard_selected().unwrap();
    };

    // chips = level(5) + card(11) = 16; each played hand adds +1 Mult first
    assert_eq!(g.calc_score(ace_hand.clone()), 32);
    assert_eq!(g.calc_score(ace_hand.clone()), 48);

    // a discard removes 1: +1 then +1 from this hand -> mult 1 + 2
    discard(&mut g);
    assert_eq!(g.calc_score(ace_hand.clone()), 48);

    // never below 0: three discards from 2 leave 0, the hand adds 1
    g.discards_remaining = g.config.discards;
    discard(&mut g);
    discard(&mut g);
    discard(&mut g);
    assert_eq!(g.calc_score(ace_hand.clone()), 32);

    // kept across rounds
    g.plays = g.config.plays;
    g.discards_remaining = g.config.discards;
    g.discarded_this_round.clear();
    assert_eq!(g.calc_score(ace_hand), 48);
}

#[test]
fn test_hit_the_road_mult_from_jacks_discarded_this_round() {
    let ace_hand = SelectHand::new(vec![Card::new(Value::Ace, Suit::Heart)])
        .best_hand()
        .unwrap();

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.money += 1000;
    g.stage = Stage::Shop();
    let mut j = Jokers::HitTheRoad(HitTheRoad::default());
    j.set_instance_id(1);
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // play a hand containing a Jack first - it lands in `g.discarded`
    // (the deck-recycling pile) but must NOT count toward HitTheRoad's
    // bonus, which is scoped to `discarded_this_round` (Discard-action
    // cards only). If the joker's filter ever regressed to reading
    // `g.discarded` instead, this Jack would wrongly inflate the count
    // and the very next assertion would fail.
    let played_jack = Card::new(Value::Jack, Suit::Diamond);
    g.available.extend(vec![played_jack]);
    g.select_card(played_jack).unwrap();
    g.play_selected().unwrap();

    let jack1 = Card::new(Value::Jack, Suit::Club);
    g.available.extend(vec![jack1]);
    g.select_card(jack1).unwrap();
    g.discard_selected().unwrap();
    // 1 jack discarded (the played jack above doesn't count):
    // X1.5: 16 * 1.5 = 24
    assert_eq!(g.calc_score(ace_hand.clone()), 24);

    let jack2 = Card::new(Value::Jack, Suit::Spade);
    g.available.extend(vec![jack2]);
    g.select_card(jack2).unwrap();
    g.discard_selected().unwrap();
    // 2 jacks discarded across 2 separate discards this round: bonus = 1 * 0.5 * 2 = 1
    // mult = 1 + 1 = 2; score = 16 * 2 = 32
    assert_eq!(g.calc_score(ace_hand), 32);
}

#[test]
fn test_yorick_mult_scales_with_run_total_cards_discarded() {
    let ace_hand = SelectHand::new(vec![Card::new(Value::Ace, Suit::Heart)])
        .best_hand()
        .unwrap();

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    // Cards discarded before Yorick is acquired don't count.
    g.total_cards_discarded = 100;
    g.money += 1000;
    g.stage = Stage::Shop();
    let mut j = Jokers::Yorick(Yorick::default());
    j.set_instance_id(1);
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    assert_eq!(g.calc_score(ace_hand.clone()), 16);

    let cards: Vec<_> = (0..3).map(|_| Card::new(Value::Two, Suit::Club)).collect();
    g.available.extend(cards.clone());
    for card in cards {
        g.select_card(card).unwrap();
    }
    g.discard_selected().unwrap();
    assert_eq!(g.joker_state_mut(1).unwrap().counter, 3.0);

    g.joker_state_mut(1).unwrap().counter = 22.0;
    assert_eq!(g.calc_score(ace_hand.clone()), 16);

    // every 23 cards discarded since acquisition: X1 more
    g.joker_state_mut(1).unwrap().counter = 23.0;
    assert_eq!(g.calc_score(ace_hand.clone()), 32);

    g.joker_state_mut(1).unwrap().counter = 46.0;
    assert_eq!(g.calc_score(ace_hand), 48);
}

#[test]
fn test_castle_keeps_its_chips_across_rounds() {
    let ace_hand = SelectHand::new(vec![Card::new(Value::Ace, Suit::Club)])
        .best_hand()
        .unwrap();

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.money += 1000;
    g.stage = Stage::Shop();
    let mut j = Jokers::Castle(Castle::default());
    j.set_instance_id(1);
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    g.joker_state_mut(1).unwrap().selector = Some(SelectorValue::Suit(Suit::Heart));

    let heart = Card::new(Value::Two, Suit::Heart);
    g.available.extend(vec![heart]);
    g.select_card(heart).unwrap();
    g.discard_selected().unwrap();

    // next round: the suit changes, the chips stay
    g.discarded_this_round.clear();
    g.joker_state_mut(1).unwrap().selector = Some(SelectorValue::Suit(Suit::Spade));
    // chips = level(5) + card(11) + 3 = 19
    assert_eq!(g.calc_score(ace_hand), 19);
}

#[test]
fn test_castle_chips_from_discarded_suit_this_round() {
    let ace_hand = SelectHand::new(vec![Card::new(Value::Ace, Suit::Club)])
        .best_hand()
        .unwrap();

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.money += 1000;
    g.stage = Stage::Shop();
    let mut j = Jokers::Castle(Castle::default());
    j.set_instance_id(1);
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    g.joker_state_mut(1).unwrap().selector = Some(SelectorValue::Suit(Suit::Heart));

    // play a Heart first - it lands in `g.discarded` (deck-recycling
    // pile) but must NOT count toward Castle's bonus, which is scoped
    // to `discarded_this_round` (Discard-action cards only). If the
    // joker's filter ever regressed to reading `g.discarded` instead,
    // this Heart would wrongly inflate the count and the assertion
    // below would fail.
    let played_heart = Card::new(Value::Five, Suit::Heart);
    g.available.extend(vec![played_heart]);
    g.select_card(played_heart).unwrap();
    g.play_selected().unwrap();

    let h1 = Card::new(Value::Two, Suit::Heart);
    let h2 = Card::new(Value::Three, Suit::Heart);
    let c1 = Card::new(Value::Four, Suit::Club); // non-matching suit, doesn't count
    g.available.extend(vec![h1, h2, c1]);
    g.select_card(h1).unwrap();
    g.select_card(h2).unwrap();
    g.select_card(c1).unwrap();
    g.discard_selected().unwrap();

    // chips = level(5) + card(11) + 2 discarded hearts (not the played
    // one) * 3 = 22; mult = level(1); score = 22
    assert_eq!(g.calc_score(ace_hand), 22);
}

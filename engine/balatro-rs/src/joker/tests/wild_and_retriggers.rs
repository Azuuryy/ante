use super::*;

#[test]
fn test_wild_counts_for_suit_jokers() {
    // Two Aces so they form a Pair, one is Wild (Spade but counts as all suits)
    let mut wild = Card::new(Value::Ace, Suit::Spade);
    wild.enhancement = Some(Enhancement::Wild);
    let heart = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![heart, wild]);

    // Pair (level 1): 10 chips, 2 mult
    // Played: Ace (11) + Ace (11) = 22 chips
    // (10 + 22) * 2 = 64
    let before = 64;

    // LustyJoker: heart (1) + wild-as-heart (1) = 2 hearts -> +6 mult
    // (10 + 22) * (2 + 6) = 32 * 8 = 256
    let after = 256;
    let j = Jokers::LustyJoker(LustyJoker::default());
    score_before_after_joker(j, hand, before, after);
}

#[test]
fn test_wild_counts_for_rough_gem() {
    let mut wild = Card::new(Value::Ace, Suit::Heart);
    wild.enhancement = Some(Enhancement::Wild);
    let hand = SelectHand::new(vec![wild]);

    let j = Jokers::RoughGem(RoughGem::default());
    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.money += 1000;
    g.stage = Stage::Shop();
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    let money_before = g.money;
    g.calc_score(hand.best_hand().unwrap());
    assert_eq!(
        g.money,
        money_before + 1,
        "Wild should count as Diamond for RoughGem"
    );
}

#[test]
fn test_hanging_chad_buy_flow() {
    // High Card Ace, alone -> first (only) card retriggers 2 extra times.
    // before: (5 + 11) * 1 = 16; after: (5 + 11*3) * 1 = 38
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]);
    let j = Jokers::HangingChad(HangingChad::default());
    score_before_after_joker(j, hand, 16, 38);
}

#[test]
fn test_hanging_chad_skips_unscored_leading_kicker() {
    // Pair of Kings with an unmatched Queen kicker played first. The Queen
    // never scores, so "first played card used in scoring" must resolve
    // to the first King, not the Queen.
    // Pair (level 1): 10 chips, 2 mult. Kings score 10 chips each.
    // before: (10 + 10 + 10) * 2 = 60
    // after: king1 retriggers (1 base + 2 from HangingChad = 3 triggers),
    // king2 doesn't: (10 + 10*3 + 10*1) * 2 = 100
    let queen = Card::new(Value::Queen, Suit::Diamond);
    let king1 = Card::new(Value::King, Suit::Heart);
    let king2 = Card::new(Value::King, Suit::Spade);
    let hand = SelectHand::new(vec![queen, king1, king2]);
    let j = Jokers::HangingChad(HangingChad::default());
    score_before_after_joker(j, hand, 60, 100);
}

#[test]
fn test_hack_buy_flow() {
    // High Card Three, alone -> retriggers since 3 is in 2-5.
    // before: (5 + 3) * 1 = 8; after: (5 + 3 + 3) * 1 = 11
    let three = Card::new(Value::Three, Suit::Heart);
    let hand = SelectHand::new(vec![three]);
    let j = Jokers::Hack(Hack::default());
    score_before_after_joker(j, hand, 8, 11);
}

#[test]
fn test_sock_and_buskin_buy_flow() {
    // High Card Jack, alone -> retriggers since Jack is a face card.
    // before: (5 + 10) * 1 = 15; after: (5 + 10 + 10) * 1 = 25
    let jack = Card::new(Value::Jack, Suit::Heart);
    let hand = SelectHand::new(vec![jack]);
    let j = Jokers::SockAndBuskin(SockAndBuskin::default());
    score_before_after_joker(j, hand, 15, 25);
}

#[test]
fn test_sock_and_buskin_pareidolia_retriggers_non_face_card() {
    // High Card Two, alone. Two isn't a face card, so SockAndBuskin alone
    // doesn't retrigger it. Once Pareidolia is also owned, it should.
    // no jokers: (5 + 2) * 1 = 7
    // + SockAndBuskin only: still 7 (not a face card)
    // + Pareidolia: (5 + 2*2) * 1 = 9 (now retriggers once)
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
    let sb = Jokers::SockAndBuskin(SockAndBuskin::default());
    g.shop.jokers.push(sb.clone());
    g.buy_joker(sb).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    assert_eq!(g.calc_score(best.clone()), 7);

    g.money += 1000;
    g.stage = Stage::Shop();
    let p = Jokers::Pareidolia(Pareidolia::default());
    g.shop.jokers.push(p.clone());
    g.buy_joker(p).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    assert_eq!(g.calc_score(best.clone()), 9);
}

#[test]
fn test_multiple_retrigger_jokers_stack() {
    // High Card Three, alone. Three qualifies for Hack (2-5)
    // and (with Pareidolia active) also for SockAndBuskin.
    // HangingChad adds a third trigger source.
    // Each trigger re-scores card's chip value (3), so total
    // trigger count directly multiplies.
    let three = Card::new(Value::Three, Suit::Heart);
    let hand = SelectHand::new(vec![three]);

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    let best = hand.best_hand().unwrap();
    // no jokers: (5 + 3) * 1 = 8
    assert_eq!(g.calc_score(best.clone()), 8);

    g.money += 1000;
    g.stage = Stage::Shop();
    let hack = Jokers::Hack(Hack::default());
    g.shop.jokers.push(hack.clone());
    g.buy_joker(hack).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    // + Hack: 2 triggers (base 1 + Hack's 1) -> (5 + 3*2) * 1 = 11
    assert_eq!(g.calc_score(best.clone()), 11);

    g.money += 1000;
    g.stage = Stage::Shop();
    let sb = Jokers::SockAndBuskin(SockAndBuskin::default());
    g.shop.jokers.push(sb.clone());
    g.buy_joker(sb).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    // + SockAndBuskin, no Pareidolia yet: Three isn't a face card -> unchanged
    assert_eq!(g.calc_score(best.clone()), 11);

    g.money += 1000;
    g.stage = Stage::Shop();
    let p = Jokers::Pareidolia(Pareidolia::default());
    g.shop.jokers.push(p.clone());
    g.buy_joker(p).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    // + Pareidolia: SockAndBuskin's check now passes too -> 3 triggers
    // (base 1 + Hack 1 + SockAndBuskin 1) -> (5 + 3*3) * 1 = 14
    assert_eq!(g.calc_score(best.clone()), 14);

    g.money += 1000;
    g.stage = Stage::Shop();
    let hc = Jokers::HangingChad(HangingChad::default());
    g.shop.jokers.push(hc.clone());
    g.buy_joker(hc).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    // + HangingChad: card is first -> +2 more -> 5 triggers total
    // (5 + 3*5) * 1 = 20
    assert_eq!(g.calc_score(best.clone()), 20);
}

#[test]
fn test_mime_buy_flow_retriggers_held_steel() {
    // Same shape/numbers as test_seal_red_retrigger_steel_held (game.rs):
    // mult 2 -> 3 (floor(2*1.5)) -> 4 (floor(3*1.5)); score = 30 * 4 = 120
    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    let king1 = Card::new(Value::King, Suit::Heart);
    let king2 = Card::new(Value::King, Suit::Diamond);
    let mut steel_king = Card::new(Value::King, Suit::Spade);
    steel_king.enhancement = Some(Enhancement::Steel);
    g.available.extend(vec![steel_king]);
    let hand = SelectHand::new(vec![king1, king2]).best_hand().unwrap();

    let score = g.calc_score(hand.clone());
    assert_eq!(score, 90);

    g.money += 1000;
    g.stage = Stage::Shop();
    let j = Jokers::Mime(Mime::default());
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // Steel held twice: 2 * 1.5 * 1.5 = 4.5; (10 + 20) * 4.5 = 135
    let score = g.calc_score(hand);
    assert_eq!(score, 135);
}

#[test]
fn test_dusk_buy_flow_retriggers_final_hand() {
    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.plays = 0;
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap();

    let score = g.calc_score(hand.clone());
    assert_eq!(score, 16);

    g.money += 1000;
    g.stage = Stage::Shop();
    let j = Jokers::Dusk(Dusk::default());
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    let score = g.calc_score(hand);
    assert_eq!(score, 27);
}

#[test]
fn test_supernova_scales_with_hand_type_plays_this_run() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap(); // HighCard

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.money += 1000;
    g.stage = Stage::Shop();
    let j = Jokers::Supernova(Supernova::default());
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // 1st play of HighCard this run: planetarium plays 0 -> 1, +1 mult
    // (5 + 11) * (1 + 1) = 32
    assert_eq!(g.calc_score(hand.clone()), 32);
    // 2nd play of HighCard this run: plays 1 -> 2, +2 mult
    // (5 + 11) * (1 + 2) = 48
    assert_eq!(g.calc_score(hand), 48);
}

#[test]
fn test_ride_the_bus_resets_on_scoring_face_card() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let no_face_hand = SelectHand::new(vec![ace]).best_hand().unwrap();

    let king = Card::new(Value::King, Suit::Heart);
    let face_hand = SelectHand::new(vec![king]).best_hand().unwrap();

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.money += 1000;
    g.stage = Stage::Shop();
    let j = Jokers::RideTheBus(RideTheBus::default());
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // no face card: streak 0 -> 1, +1 mult -> (5 + 11) * (1 + 1) = 32
    assert_eq!(g.calc_score(no_face_hand.clone()), 32);
    // no face card again: streak 1 -> 2, +2 mult -> (5 + 11) * (1 + 2) = 48
    assert_eq!(g.calc_score(no_face_hand), 48);
    // scoring face card resets the streak, no bonus this hand
    // (5 + 10) * 1 = 15
    assert_eq!(g.calc_score(face_hand), 15);
}

#[test]
fn test_card_sharp_triples_mult_on_repeat_hand_type_same_round() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let hand = SelectHand::new(vec![ace]).best_hand().unwrap(); // HighCard

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.money += 1000;
    g.stage = Stage::Shop();
    let j = Jokers::CardSharp(CardSharp::default());
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // 1st play of HighCard this round: not yet played -> no bonus
    assert_eq!(g.calc_score(hand.clone()), 16);
    // 2nd play of HighCard, same round: already played -> X3 mult
    // (5 + 11) * (1 * 3) = 48
    assert_eq!(g.calc_score(hand.clone()), 48);

    g.clear_blind();

    // new round: the played-this-round set was cleared -> no bonus again
    assert_eq!(g.calc_score(hand), 16);
}

#[test]
fn test_obelisk_scales_mult_for_hands_not_matching_most_played_type() {
    let ace = Card::new(Value::Ace, Suit::Heart);
    let high_card_hand = SelectHand::new(vec![ace]).best_hand().unwrap(); // HighCard

    let four_aces = Card::new(Value::Ace, Suit::Club);
    let four_of_kind_hand = SelectHand::new(vec![four_aces, four_aces, four_aces, four_aces])
        .best_hand()
        .unwrap(); // FourOfAKind

    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };

    // Establish HighCard as the run's most-played type before Obelisk exists.
    for _ in 0..3 {
        g.calc_score(high_card_hand.clone());
    }

    g.money += 1000;
    g.stage = Stage::Shop();
    let j = Jokers::Obelisk(Obelisk::default());
    g.shop.jokers.push(j.clone());
    g.buy_joker(j).unwrap();
    g.stage = Stage::Blind(Blind::Small);

    // FourOfAKind != most-played (HighCard, 3 plays) -> X1.2
    // floor(104 * 7 * 1.2) = floor(873.6) = 873
    assert_eq!(g.calc_score(four_of_kind_hand.clone()), 873);
    // still != most-played (HighCard now leads 3 to 2) -> X1.4
    // floor(104 * 7 * 1.4) = floor(1019.2) = 1019
    assert_eq!(g.calc_score(four_of_kind_hand), 1019);
    // HighCard == most-played (now 4 plays, still highest) -> streak resets
    // (5 + 11) * 1 = 16
    assert_eq!(g.calc_score(high_card_hand), 16);
}

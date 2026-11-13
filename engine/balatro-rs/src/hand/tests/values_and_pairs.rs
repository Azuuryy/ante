use super::*;

#[test]
fn test_values() {
    let c3 = Card::new(Value::Two, Suit::Heart);
    let c4 = Card::new(Value::Three, Suit::Diamond);
    let c5 = Card::new(Value::Jack, Suit::Heart);
    let c1 = Card::new(Value::King, Suit::Heart);
    let c2 = Card::new(Value::Ace, Suit::Spade);

    let hand = SelectHand::new(vec![c1, c2, c3, c4, c5]);
    let values = hand.values();

    // Should have 5 values
    assert_eq!(values.len(), 5);

    // Expect sorted (2, 3, J, K, A)
    assert_eq!(values[0], Value::Two);
    assert_eq!(values[1], Value::Three);
    assert_eq!(values[2], Value::Jack);
    assert_eq!(values[3], Value::King);
    assert_eq!(values[4], Value::Ace);
}

#[test]
fn test_values_freq() {
    let c1 = Card::new(Value::Two, Suit::Heart);
    let c2 = Card::new(Value::Three, Suit::Diamond);
    let c3 = Card::new(Value::Four, Suit::Heart);
    let c4 = Card::new(Value::King, Suit::Heart);
    let c5 = Card::new(Value::King, Suit::Spade);

    let hand = SelectHand::new(vec![c1, c2, c3, c4, c5]);
    let freq = hand.values_freq();

    // Should have 4 values (K, 2, 3, 4)
    assert_eq!(freq.len(), 4);

    // Expect 2 kings and 1 each of 2, 3, 4
    assert_eq!(freq.get(&Value::King).unwrap().len(), 2);
    assert_eq!(freq.get(&Value::Two).unwrap().len(), 1);
    assert_eq!(freq.get(&Value::Three).unwrap().len(), 1);
    assert_eq!(freq.get(&Value::Four).unwrap().len(), 1);

    // No extra cards
    assert_eq!(freq.get(&Value::Five), None);
    assert_eq!(freq.get(&Value::Nine), None);

    // Can also check the cards in the vec are as expected
    assert_eq!(freq.get(&Value::King).unwrap()[0].value, Value::King);
    assert_eq!(freq.get(&Value::King).unwrap()[1].value, Value::King);
    assert_eq!(freq.get(&Value::Two).unwrap()[0].value, Value::Two);
    assert_eq!(freq.get(&Value::Three).unwrap()[0].value, Value::Three);
    assert_eq!(freq.get(&Value::Four).unwrap()[0].value, Value::Four);

    // Check ordered by value
    assert_eq!(freq.into_iter().next().unwrap().0, Value::King)
}

#[test]
fn test_suits_freq() {
    let c1 = Card::new(Value::King, Suit::Heart);
    let c2 = Card::new(Value::King, Suit::Spade);
    let c3 = Card::new(Value::Two, Suit::Heart);
    let c4 = Card::new(Value::Three, Suit::Diamond);
    let c5 = Card::new(Value::Four, Suit::Heart);

    let hand = SelectHand::new(vec![c1, c2, c3, c4, c5]);
    let freq = hand.suits_freq();

    // Should have 3 values (heart, spade, diamond)
    assert_eq!(freq.len(), 3);

    // Expect 3 hearts and 1 each of spade and diamond
    assert_eq!(freq.get(&Suit::Heart).unwrap().len(), 3);
    assert_eq!(freq.get(&Suit::Spade).unwrap().len(), 1);
    assert_eq!(freq.get(&Suit::Diamond).unwrap().len(), 1);

    // No clubs to be found
    assert_eq!(freq.get(&Suit::Club), None);

    // Can also check the cards in the vec are as expected
    assert_eq!(freq.get(&Suit::Heart).unwrap()[0].suit, Suit::Heart);
    assert_eq!(freq.get(&Suit::Heart).unwrap()[1].suit, Suit::Heart);
    assert_eq!(freq.get(&Suit::Heart).unwrap()[2].suit, Suit::Heart);
    assert_eq!(freq.get(&Suit::Spade).unwrap()[0].suit, Suit::Spade);
    assert_eq!(freq.get(&Suit::Diamond).unwrap()[0].suit, Suit::Diamond);
}

#[test]
fn test_best_hand() {
    let c1 = Card::new(Value::Ace, Suit::Heart);
    let c2 = Card::new(Value::Two, Suit::Heart);
    let c3 = Card::new(Value::Three, Suit::Diamond);

    // Best hand is flush five (Ah, Ah, Ah, Ah, Ah)
    let hand = SelectHand::new(vec![c1, c1, c1, c1, c1]);
    let best = hand.best_hand().expect("is best hand");
    assert_eq!(best.rank, HandRank::FlushFive);
    assert_eq!(best.hand.len(), 5);

    // 4ok is better than flush (Ah, Ah, Ah, Ah, 2h)
    let hand = SelectHand::new(vec![c1, c1, c1, c1, c2]);
    let best = hand.best_hand().expect("is best hand");
    assert_eq!(best.clone().rank, HandRank::FourOfAKind);
    assert_eq!(best.hand.len(), 4);

    // Two pair is better than pair (Ah, Ah, 2h, 2h, 3d)
    let hand = SelectHand::new(vec![c1, c1, c2, c2, c3]);
    let best = hand.best_hand().expect("is best hand");
    assert_eq!(best.clone().rank, HandRank::TwoPair);
    assert_eq!(best.hand.len(), 4);

    // At worst, we get a high card (Ah, 2h, 3d)
    let hand = SelectHand::new(vec![c1, c2, c3]);
    let best = hand.best_hand().expect("is best hand");
    assert_eq!(best.clone().rank, HandRank::HighCard);
    assert_eq!(best.hand.len(), 1);
}

#[test]
fn test_highcard() {
    let c1 = Card::new(Value::Ace, Suit::Heart);
    let c2 = Card::new(Value::King, Suit::Heart);
    let c3 = Card::new(Value::Three, Suit::Diamond);
    let c4 = Card::new(Value::Four, Suit::Diamond);
    let c5 = Card::new(Value::Five, Suit::Diamond);
    let c6 = Card::new(Value::Six, Suit::Diamond);

    // Valid 5 (A, K, 3, 4, 5)
    let hand = SelectHand::new(vec![c1, c2, c3, c4, c5]);
    let hc = hand.is_highcard();
    assert_eq!(hc.clone().unwrap().len(), 1);
    assert_eq!(hc.unwrap().0[0].value, Value::Ace);

    // Valid 5 (K, A, 3, 4, 5)
    let hand = SelectHand::new(vec![c2, c1, c3, c4, c5]);
    let hc = hand.is_highcard();
    assert_eq!(hc.clone().unwrap().len(), 1);
    assert_eq!(hc.unwrap().0[0].value, Value::Ace);

    // Valid 5 (K, 3, 4, 5, 6)
    let hand = SelectHand::new(vec![c2, c3, c4, c5, c6]);
    let hc = hand.is_highcard();
    assert_eq!(hc.clone().unwrap().len(), 1);
    assert_eq!(hc.unwrap().0[0].value, Value::King);

    // Valid 4 (K, 3, 4, 5)
    let hand = SelectHand::new(vec![c2, c3, c4, c5]);
    let hc = hand.is_highcard();
    assert_eq!(hc.clone().unwrap().len(), 1);
    assert_eq!(hc.unwrap().0[0].value, Value::King);

    // Valid 3 (K, 3, 4)
    let hand = SelectHand::new(vec![c2, c3, c4]);
    let hc = hand.is_highcard();
    assert_eq!(hc.clone().unwrap().len(), 1);
    assert_eq!(hc.unwrap().0[0].value, Value::King);

    // Valid 2 (K, 3)
    let hand = SelectHand::new(vec![c2, c3]);
    let hc = hand.is_highcard();
    assert_eq!(hc.clone().unwrap().len(), 1);
    assert_eq!(hc.unwrap().0[0].value, Value::King);

    // Valid 1 (K)
    let hand = SelectHand::new(vec![c2]);
    let hc = hand.is_highcard();
    assert_eq!(hc.clone().unwrap().len(), 1);
    assert_eq!(hc.unwrap().0[0].value, Value::King);
}

#[test]
fn test_highcard_preserves_order() {
    let ah = Card::new(Value::Ace, Suit::Heart);
    let as_ = Card::new(Value::Ace, Suit::Spade);
    let two = Card::new(Value::Two, Suit::Diamond);

    // Two aces tie for high card — order should follow input order.
    let hand = SelectHand::new(vec![ah, two, as_]);
    let hc = hand.is_highcard().unwrap();
    assert_eq!(hc.cards(), vec![ah, as_]);
}

#[test]
fn test_pair() {
    let c1 = Card::new(Value::King, Suit::Heart);
    let c2 = Card::new(Value::King, Suit::Diamond);
    let c3 = Card::new(Value::Three, Suit::Diamond);
    let c4 = Card::new(Value::Four, Suit::Diamond);
    let c5 = Card::new(Value::Five, Suit::Diamond);
    let c6 = Card::new(Value::Six, Suit::Diamond);

    // Valid 5 (K, K, 3, 4, 5)
    let hand = SelectHand::new(vec![c1, c2, c3, c4, c5]);
    let is_2 = hand.is_pair();
    assert_eq!(is_2.unwrap().len(), 2);

    // Valid 4 (K, K, 3, 4)
    let hand = SelectHand::new(vec![c1, c2, c3, c4]);
    let is_2 = hand.is_pair();
    assert_eq!(is_2.unwrap().len(), 2);

    // Valid 3 (K, K, 3)
    let hand = SelectHand::new(vec![c1, c2, c3]);
    let is_2 = hand.is_pair();
    assert_eq!(is_2.unwrap().len(), 2);

    // Valid 2 (K, K)
    let hand = SelectHand::new(vec![c1, c2]);
    let is_2 = hand.is_pair();
    assert_eq!(is_2.unwrap().len(), 2);

    // Invalid 1 (K)
    let hand = SelectHand::new(vec![c1]);
    let is_2 = hand.is_pair();
    assert_eq!(is_2, None);

    // Invalid 2 (K, 3)
    let hand = SelectHand::new(vec![c1, c3]);
    let is_2 = hand.is_pair();
    assert_eq!(is_2, None);

    // Invalid 3 (K, 3, 4)
    let hand = SelectHand::new(vec![c1, c3, c4]);
    let is_2 = hand.is_pair();
    assert_eq!(is_2, None);

    // Invalid 4 (K, 3, 4, 5)
    let hand = SelectHand::new(vec![c1, c3, c4, c5]);
    let is_2 = hand.is_pair();
    assert_eq!(is_2, None);

    // Invalid 5 (K, 3, 4, 5, 6)
    let hand = SelectHand::new(vec![c1, c3, c4, c5, c6]);
    let is_2 = hand.is_pair();
    assert_eq!(is_2, None);
}

#[test]
fn test_pair_preserves_order() {
    let kh = Card::new(Value::King, Suit::Heart);
    let ks = Card::new(Value::King, Suit::Spade);
    let ah = Card::new(Value::Ace, Suit::Heart);
    let two = Card::new(Value::Two, Suit::Diamond);

    // Kings arranged after the ace kicker, result follows input order.
    let hand = SelectHand::new(vec![ah, kh, two, ks]);
    let pair = hand.is_pair().unwrap();
    assert_eq!(pair.cards(), vec![kh, ks]);
}

#[test]
fn test_two_pair() {
    let c1 = Card::new(Value::King, Suit::Heart);
    let c2 = Card::new(Value::King, Suit::Spade);
    let c3 = Card::new(Value::Four, Suit::Diamond);
    let c4 = Card::new(Value::Four, Suit::Heart);
    let not1 = Card::new(Value::Two, Suit::Heart);
    let not2 = Card::new(Value::Three, Suit::Heart);

    // Valid 5 (K, K, 4, 4, 2)
    let hand = SelectHand::new(vec![c1, c2, c3, c4, not1]);
    let tp = hand.is_two_pair();
    assert_eq!(tp.unwrap().len(), 4);

    // Valid 4 (K, K, 4, 4)
    let hand = SelectHand::new(vec![c1, c2, c3, c4]);
    let tp = hand.is_two_pair();
    assert_eq!(tp.unwrap().len(), 4);

    // Invalid 5 (K, K, K, K, 2)
    let hand = SelectHand::new(vec![c1, c1, c2, c2, not1]);
    let tp = hand.is_two_pair();
    assert_eq!(tp, None);

    // Invalid 5 (K, 4, 3, 2, 2)
    let hand = SelectHand::new(vec![c1, c4, not1, not2, not2]);
    let tp = hand.is_two_pair();
    assert_eq!(tp, None);

    // Invalid 5 (K, K, 4, 3, 2)
    let hand = SelectHand::new(vec![c1, c1, c4, not1, not2]);
    let tp = hand.is_two_pair();
    assert_eq!(tp, None);

    // Invalid 4 (K, K, 4, 2)
    let hand = SelectHand::new(vec![c1, c2, c4, not1]);
    let tp = hand.is_two_pair();
    assert_eq!(tp, None);
}

#[test]
fn test_two_pair_preserves_order() {
    let two_h = Card::new(Value::Two, Suit::Heart);
    let two_s = Card::new(Value::Two, Suit::Spade);
    let kh = Card::new(Value::King, Suit::Heart);
    let ks = Card::new(Value::King, Suit::Spade);

    // Old code always returned the higher-valued pair first regardless
    // of arrangement, this pins the fix.
    let hand = SelectHand::new(vec![two_h, two_s, kh, ks]);
    let tp = hand.is_two_pair().unwrap();
    assert_eq!(tp.cards(), vec![two_h, two_s, kh, ks]);

    // Interleaved arrangement should come out interleaved.
    let hand = SelectHand::new(vec![two_h, kh, two_s, ks]);
    let tp = hand.is_two_pair().unwrap();
    assert_eq!(tp.cards(), vec![two_h, kh, two_s, ks]);
}

#[test]
fn test_three_of_kind() {
    let c1 = Card::new(Value::King, Suit::Heart);
    let c2 = Card::new(Value::King, Suit::Spade);
    let c3 = Card::new(Value::King, Suit::Heart);
    let not1 = Card::new(Value::Ace, Suit::Heart);
    let not2 = Card::new(Value::Two, Suit::Heart);

    // Valid 5 (K, K, K, A, 2)
    let hand = SelectHand::new(vec![c1, c2, c3, not1, not2]);
    let is_3 = hand.is_three_of_kind();
    assert_eq!(is_3.unwrap().len(), 3);

    // Valid 4 (K, K, K, A)
    let hand = SelectHand::new(vec![c1, c2, c3, not1]);
    let is_3 = hand.is_three_of_kind();
    assert_eq!(is_3.unwrap().len(), 3);

    // Valid 3 (K, K, K)
    let hand = SelectHand::new(vec![c1, c2, c3]);
    let is_3 = hand.is_three_of_kind();
    assert_eq!(is_3.unwrap().len(), 3);

    // Invalid 3 (K, K, A)
    let hand = SelectHand::new(vec![c1, c2, not1]);
    let is_3 = hand.is_three_of_kind();
    assert_eq!(is_3, None);

    // Invalid 4 (K, K, A, A),
    let hand = SelectHand::new(vec![c1, c2, not1, not1]);
    let is_3 = hand.is_three_of_kind();
    assert_eq!(is_3, None);

    // Invalid 5 (K, K, A, A, 2),
    let hand = SelectHand::new(vec![c1, c2, not1, not1, not2]);
    let is_3 = hand.is_three_of_kind();
    assert_eq!(is_3, None);

    // Invalid 2 (K, K)
    let hand = SelectHand::new(vec![c1, c2]);
    let is_3 = hand.is_three_of_kind();
    assert_eq!(is_3, None);
}

use super::*;

#[test]
fn test_three_of_kind_preserves_order() {
    let kh = Card::new(Value::King, Suit::Heart);
    let ks = Card::new(Value::King, Suit::Spade);
    let kd = Card::new(Value::King, Suit::Diamond);
    let ah = Card::new(Value::Ace, Suit::Heart);

    // Kings not adjacent in the input.
    let hand = SelectHand::new(vec![kh, ah, ks, kd]);
    let is_3 = hand.is_three_of_kind().unwrap();
    assert_eq!(is_3.cards(), vec![kh, ks, kd]);
}

#[test]
fn test_straight() {
    let c1 = Card::new(Value::Ace, Suit::Heart);
    let c2 = Card::new(Value::Two, Suit::Heart);
    let c3 = Card::new(Value::Three, Suit::Heart);
    let c4 = Card::new(Value::Four, Suit::Heart);
    let c5 = Card::new(Value::Five, Suit::Heart);
    let c6 = Card::new(Value::Six, Suit::Diamond);
    let c7 = Card::new(Value::Seven, Suit::Diamond);

    // Valid 5 (2, 3, 4 ,5 ,6)
    let hand = SelectHand::new(vec![c2, c3, c4, c5, c6]);
    let straight = hand.is_straight();
    assert_eq!(straight.unwrap().len(), 5);

    // Valid 5 with low ace (A, 2, 3, 4 ,5)
    let hand = SelectHand::new(vec![c1, c2, c3, c4, c5]);
    let straight = hand.is_straight();
    assert_eq!(straight.unwrap().len(), 5);

    // Invalid 5 (2, 3, 4, 5, 7)
    let hand = SelectHand::new(vec![c2, c3, c4, c5, c7]);
    let straight = hand.is_straight();
    assert_eq!(straight, None);

    // Invalid 5 with low ace (A, 2, 3, 4, 7)
    let hand = SelectHand::new(vec![c1, c2, c3, c4, c7]);
    let straight = hand.is_straight();
    assert_eq!(straight, None);

    // Invalid 4 (2, 3, 4, 5)
    let hand = SelectHand::new(vec![c2, c3, c4, c5]);
    let straight = hand.is_straight();
    assert_eq!(straight, None);
}

#[test]
fn test_flush() {
    let c1 = Card::new(Value::King, Suit::Heart);
    let c2 = Card::new(Value::Queen, Suit::Heart);
    let c3 = Card::new(Value::Jack, Suit::Heart);
    let c4 = Card::new(Value::Seven, Suit::Heart);
    let c5 = Card::new(Value::Eight, Suit::Heart);
    let not = Card::new(Value::Ace, Suit::Diamond);

    // Valid 5 (h, h, h, h, h)
    let hand = SelectHand::new(vec![c1, c2, c3, c4, c5]);
    let flush = hand.is_flush();
    assert_eq!(flush.unwrap().len(), 5);

    // Valid 5 from 7 cards (h, h, h, h, h, d, d)
    let hand = SelectHand::new(vec![c1, c2, c3, c4, c5, not, not]);
    let flush = hand.is_flush();
    assert_eq!(flush.unwrap().len(), 5);

    // Invalid 5 (h, h, h, h, d)
    let hand = SelectHand::new(vec![c1, c2, c3, c4, not]);
    let flush = hand.is_flush();
    assert_eq!(flush, None);

    // Invalid 4 (h, h, h, h)
    let hand = SelectHand::new(vec![c1, c2, c3, c4]);
    let flush = hand.is_flush();
    assert_eq!(flush, None);
}

#[test]
fn test_fullhouse() {
    let c1 = Card::new(Value::King, Suit::Heart);
    let c2 = Card::new(Value::King, Suit::Spade);
    let c3 = Card::new(Value::King, Suit::Heart);
    let c4 = Card::new(Value::Four, Suit::Diamond);
    let c5 = Card::new(Value::Four, Suit::Heart);
    let not1 = Card::new(Value::Two, Suit::Heart);
    let not2 = Card::new(Value::Three, Suit::Heart);

    // Valid 5 (K, K, K, 4, 4)
    let hand = SelectHand::new(vec![c1, c2, c3, c4, c5]);
    let is_fh = hand.is_fullhouse();
    assert_eq!(is_fh.unwrap().len(), 5);

    // Valid 5 from 7 cards (K, K, K, 4, 4, 2, 3)
    let hand = SelectHand::new(vec![c1, c2, c3, c4, c5, not1, not2]);
    let is_fh = hand.is_fullhouse();
    assert_eq!(is_fh.unwrap().len(), 5);

    // Invalid 5 (K, K, K, K, 2)
    let hand = SelectHand::new(vec![c1, c2, c3, c3, not1]);
    let is_fh = hand.is_fullhouse();
    assert_eq!(is_fh, None);

    // Invalid 5 (K, K, 4, 4, 2)
    let hand = SelectHand::new(vec![c1, c2, c4, c5, not1]);
    let is_fh = hand.is_fullhouse();
    assert_eq!(is_fh, None);

    // Invalid 4 (K, K, 4, 4)
    let hand = SelectHand::new(vec![c1, c2, c4, c5]);
    let is_fh = hand.is_fullhouse();
    assert_eq!(is_fh, None);
}

#[test]
fn test_fullhouse_preserves_order() {
    let two_h = Card::new(Value::Two, Suit::Heart);
    let two_s = Card::new(Value::Two, Suit::Spade);
    let kh = Card::new(Value::King, Suit::Heart);
    let ks = Card::new(Value::King, Suit::Spade);
    let kd = Card::new(Value::King, Suit::Diamond);

    // Pair arranged before the trips, old code always returned the
    // three-of-a-kind group first regardless of arrangement.
    let hand = SelectHand::new(vec![two_h, two_s, kh, ks, kd]);
    let is_fh = hand.is_fullhouse().unwrap();
    assert_eq!(is_fh.cards(), vec![two_h, two_s, kh, ks, kd]);
}

#[test]
fn test_four_of_kind() {
    let c1 = Card::new(Value::King, Suit::Heart);
    let c2 = Card::new(Value::King, Suit::Spade);
    let c3 = Card::new(Value::King, Suit::Heart);
    let c4 = Card::new(Value::King, Suit::Diamond);
    let not = Card::new(Value::Ace, Suit::Heart);

    // Valid 4 (K, K, K, K)
    let hand = SelectHand::new(vec![c1, c2, c3, c4, not]);
    let is_4 = hand.is_four_of_kind();
    assert_eq!(is_4.unwrap().len(), 4);

    // Valid 4 from 7 cards (K, K, K, K, A, A, A)
    let hand = SelectHand::new(vec![c1, c2, c3, c4, not, not, not]);
    let is_4 = hand.is_four_of_kind();
    assert_eq!(is_4.unwrap().len(), 4);

    // Invalid 4 (K, K, K, A)
    let hand = SelectHand::new(vec![c1, c2, c3, not]);
    let is_4 = hand.is_four_of_kind();
    assert_eq!(is_4, None);

    // Invalid 3 (K, K, K)
    let hand = SelectHand::new(vec![c1, c2, c3]);
    let is_4 = hand.is_four_of_kind();
    assert_eq!(is_4, None);
}

#[test]
fn test_four_of_kind_preserves_order() {
    let kh = Card::new(Value::King, Suit::Heart);
    let ks = Card::new(Value::King, Suit::Spade);
    let kd = Card::new(Value::King, Suit::Diamond);
    let kc = Card::new(Value::King, Suit::Club);
    let ah = Card::new(Value::Ace, Suit::Heart);

    let hand = SelectHand::new(vec![kh, ah, ks, kd, kc]);
    let is_4 = hand.is_four_of_kind().unwrap();
    assert_eq!(is_4.cards(), vec![kh, ks, kd, kc]);
}

#[test]
fn test_straight_flush() {
    let c1 = Card::new(Value::Ace, Suit::Heart);
    let c2 = Card::new(Value::Two, Suit::Heart);
    let c3 = Card::new(Value::Three, Suit::Heart);
    let c4 = Card::new(Value::Four, Suit::Heart);
    let c5 = Card::new(Value::Five, Suit::Heart);
    let c6 = Card::new(Value::Six, Suit::Heart);
    let not1 = Card::new(Value::Seven, Suit::Heart);
    let not2 = Card::new(Value::Six, Suit::Diamond);

    // Valid 5 (2h, 3h, 4h, 5h ,6h)
    let hand = SelectHand::new(vec![c2, c3, c4, c5, c6]);
    let sf = hand.is_straight_flush();
    assert_eq!(sf.unwrap().len(), 5);

    // Valid 5 with low ace (Ah, 2h, 3h, 4h, 5h)
    let hand = SelectHand::new(vec![c1, c2, c3, c4, c5]);
    let sf = hand.is_straight_flush();
    assert_eq!(sf.unwrap().len(), 5);

    // Invalid 5, wrong value (2h, 3h, 4h, 5h, 7h)
    let hand = SelectHand::new(vec![c2, c3, c4, c5, not1]);
    let sf = hand.is_straight_flush();
    assert_eq!(sf, None);

    // Invalid 5, wrong suit (2h, 3h, 4h, 5h, 6d)
    let hand = SelectHand::new(vec![c2, c3, c4, c5, not2]);
    let sf = hand.is_straight_flush();
    assert_eq!(sf, None);

    // Invalid 4 (2h, 3h, 4h, 5h)
    let hand = SelectHand::new(vec![c2, c3, c4, c5]);
    let sf = hand.is_straight_flush();
    assert_eq!(sf, None);
}

#[test]
fn test_royal_flush() {
    let c1 = Card::new(Value::Ten, Suit::Spade);
    let c2 = Card::new(Value::Jack, Suit::Spade);
    let c3 = Card::new(Value::Queen, Suit::Spade);
    let c4 = Card::new(Value::King, Suit::Spade);
    let c5 = Card::new(Value::Ace, Suit::Spade);
    let not1 = Card::new(Value::Nine, Suit::Spade);
    let not2 = Card::new(Value::Ace, Suit::Diamond);

    // Valid 5 (10s, Js, Qs, Ks, As)
    let hand = SelectHand::new(vec![c1, c2, c3, c4, c5]);
    let rf = hand.is_royal_flush();
    assert_eq!(rf.unwrap().len(), 5);

    // Valid 5, scrambled input order (Js, 10s, Ks, Qs, As)
    let hand = SelectHand::new(vec![c2, c1, c4, c3, c5]);
    let rf = hand.is_royal_flush();
    assert_eq!(rf.unwrap().len(), 5);

    // Invalid 5, wrong value (9s, Js, Qs, Ks, As)
    let hand = SelectHand::new(vec![not1, c2, c3, c4, c5]);
    let rf = hand.is_royal_flush();
    assert_eq!(rf, None);

    // Invalid 5, wrong suit (10s, Js, Qs, Ks, Ad)
    let hand = SelectHand::new(vec![c1, c2, c3, c4, not2]);
    let rf = hand.is_royal_flush();
    assert_eq!(rf, None);

    // Invalid 4 (2h, 3h, 4h, 5h)
    let hand = SelectHand::new(vec![c2, c3, c4, c5]);
    let rf = hand.is_royal_flush();
    assert_eq!(rf, None);
}

#[test]
fn test_five_of_kind() {
    let c1 = Card::new(Value::King, Suit::Heart);
    let c2 = Card::new(Value::King, Suit::Spade);
    let c3 = Card::new(Value::King, Suit::Heart);
    let c4 = Card::new(Value::King, Suit::Diamond);
    let c5 = Card::new(Value::King, Suit::Heart);
    let not = Card::new(Value::Ace, Suit::Heart);

    // Valid 5 (K, K, K, K, K)
    let hand = SelectHand::new(vec![c1, c2, c3, c4, c5]);
    let is_5 = hand.is_five_of_kind();
    assert_eq!(is_5.unwrap().len(), 5);

    // Valid 5 from 7 cards (K, K, K, K, K, A, A)
    let hand = SelectHand::new(vec![c1, c2, c3, c4, c5, not, not]);
    let is_5 = hand.is_five_of_kind();
    assert_eq!(is_5.unwrap().len(), 5);

    // Invalid 5 (K, K, K, K, A)
    let hand = SelectHand::new(vec![c1, c2, c3, c4, not]);
    let is_5 = hand.is_five_of_kind();
    assert_eq!(is_5, None);

    // Invalid 4 (K, K, K, K)
    let hand = SelectHand::new(vec![c1, c2, c3, c4]);
    let is_5 = hand.is_five_of_kind();
    assert_eq!(is_5, None);
}

#[test]
fn test_five_of_kind_preserves_order() {
    let k1 = Card::new(Value::King, Suit::Heart);
    let k2 = Card::new(Value::King, Suit::Spade);
    let k3 = Card::new(Value::King, Suit::Diamond);
    let k4 = Card::new(Value::King, Suit::Club);
    let k5 = Card::new(Value::King, Suit::Heart);
    let ah = Card::new(Value::Ace, Suit::Heart);

    let hand = SelectHand::new(vec![k1, ah, k2, k3, k4, k5]);
    let is_5 = hand.is_five_of_kind().unwrap();
    assert_eq!(is_5.cards(), vec![k1, k2, k3, k4, k5]);
}

#[test]
fn test_flush_house() {
    let c1 = Card::new(Value::King, Suit::Heart);
    let c2 = Card::new(Value::King, Suit::Heart);
    let c3 = Card::new(Value::King, Suit::Heart);
    let c4 = Card::new(Value::Ace, Suit::Heart);
    let c5 = Card::new(Value::Ace, Suit::Heart);
    let not1 = Card::new(Value::Two, Suit::Heart);
    let not2 = Card::new(Value::Ace, Suit::Diamond);

    // Valid 5 (Kh, Kh, Kh, Ah, Ah)
    let hand = SelectHand::new(vec![c1, c2, c3, c4, c5]);
    let fh = hand.is_flush_house();
    assert_eq!(fh.unwrap().len(), 5);

    // Invalid 5 (Kh, Kh, Kh, Ah, 2h)
    let hand = SelectHand::new(vec![c1, c2, c3, c4, not1]);
    let fh = hand.is_flush_house();
    assert_eq!(fh, None);

    // Invalid 5 (Kh, Kh, Kh, Ah, Ad)
    let hand = SelectHand::new(vec![c1, c2, c3, c4, not2]);
    let fh = hand.is_flush_house();
    assert_eq!(fh, None);

    // Invalid 4 (Kh, Kh, Kh, Ah)
    let hand = SelectHand::new(vec![c1, c2, c3, c4]);
    let fh = hand.is_flush_house();
    assert_eq!(fh, None);
}

#[test]
fn test_flush_with_wild() {
    // 4 hearts + 1 Wild = flush
    let c1 = Card::new(Value::King, Suit::Heart);
    let c2 = Card::new(Value::Queen, Suit::Heart);
    let c3 = Card::new(Value::Jack, Suit::Heart);
    let c4 = Card::new(Value::Seven, Suit::Heart);
    let mut wild = Card::new(Value::Two, Suit::Diamond);
    wild.enhancement = Some(Enhancement::Wild);

    let hand = SelectHand::new(vec![c1, c2, c3, c4, wild]);
    assert!(hand.is_flush().is_some());
    assert_eq!(hand.is_flush().unwrap().len(), 5);
}

#[test]
fn test_flush_wild_insufficient() {
    // 3 hearts + 1 diamond + 1 Wild = not a flush
    let c1 = Card::new(Value::King, Suit::Heart);
    let c2 = Card::new(Value::Queen, Suit::Heart);
    let c3 = Card::new(Value::Jack, Suit::Heart);
    let c4 = Card::new(Value::Ace, Suit::Diamond);
    let mut wild = Card::new(Value::Two, Suit::Club);
    wild.enhancement = Some(Enhancement::Wild);

    let hand = SelectHand::new(vec![c1, c2, c3, c4, wild]);
    assert!(hand.is_flush().is_none());
}

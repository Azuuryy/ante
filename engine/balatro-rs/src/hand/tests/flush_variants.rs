use super::*;

#[test]
fn test_flush_two_wilds() {
    // 3 hearts + 2 Wilds = flush
    let c1 = Card::new(Value::King, Suit::Heart);
    let c2 = Card::new(Value::Queen, Suit::Heart);
    let c3 = Card::new(Value::Jack, Suit::Heart);
    let mut wild1 = Card::new(Value::Two, Suit::Diamond);
    wild1.enhancement = Some(Enhancement::Wild);
    let mut wild2 = Card::new(Value::Three, Suit::Spade);
    wild2.enhancement = Some(Enhancement::Wild);

    let hand = SelectHand::new(vec![c1, c2, c3, wild1, wild2]);
    assert!(hand.is_flush().is_some());
}

#[test]
fn test_flush_five() {
    let c1 = Card::new(Value::King, Suit::Heart);
    let c2 = Card::new(Value::King, Suit::Heart);
    let c3 = Card::new(Value::King, Suit::Heart);
    let c4 = Card::new(Value::King, Suit::Heart);
    let c5 = Card::new(Value::King, Suit::Heart);
    let not1 = Card::new(Value::Two, Suit::Heart);
    let not2 = Card::new(Value::King, Suit::Diamond);

    // Valid 5 (Kh, Kh, Kh, Kh, Kh)
    let hand = SelectHand::new(vec![c1, c2, c3, c4, c5]);
    let ff = hand.is_flush_five();
    assert_eq!(ff.unwrap().len(), 5);

    // Invalid 5 (Kh, Kh, Kh, Kh, 2h)
    let hand = SelectHand::new(vec![c1, c2, c3, c4, not1]);
    let ff = hand.is_flush_five();
    assert_eq!(ff, None);

    // Invalid 5 (Kh, Kh, Kh, Kh, Kd)
    let hand = SelectHand::new(vec![c1, c2, c3, c4, not2]);
    let ff = hand.is_flush_five();
    assert_eq!(ff, None);

    // Invalid 4 (Kh, Kh, Kh, Kh)
    let hand = SelectHand::new(vec![c1, c2, c3, c4]);
    let ff = hand.is_flush_five();
    assert_eq!(ff, None);
}

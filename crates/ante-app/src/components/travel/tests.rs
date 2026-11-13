use super::*;

fn ready(offset: Vec2) -> Travel {
    let mut travel = Travel::new(CardKey(1), false, 0);
    travel.ready = true;
    travel.offset = offset;
    travel
}

#[test]
fn a_card_settles_on_its_slot() {
    let mut travel = ready(Vec2::new(-400.0, 250.0));
    for _ in 0..120 {
        travel.step(1.0 / 60.0);
    }
    assert_eq!(travel.offset, Vec2::ZERO);
}

#[test]
fn speed_is_capped_at_seventy_units_per_second() {
    let mut travel = ready(Vec2::new(-100.0 * TILE, 0.0));
    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        travel.step(dt);
        assert!(travel.velocity.length() <= 70.0 * TILE * dt + 1e-3);
    }
}

#[test]
fn a_delayed_card_waits_before_moving() {
    let mut travel = ready(Vec2::new(-200.0, 0.0));
    travel.delay = 0.1;
    travel.step(0.05);
    assert_eq!(travel.offset, Vec2::new(-200.0, 0.0));
    travel.step(0.06);
    travel.step(0.016);
    assert!(travel.offset.x > -200.0);
}

#[test]
fn a_discarded_card_is_done_once_it_reaches_the_pile() {
    let mut travel = ready(Vec2::ZERO);
    travel.discard = Vec2::new(300.0, -DISCARD_RISE);
    assert!(!travel.discarded());
    travel.discard(0.0);
    for _ in 0..240 {
        travel.step(1.0 / 60.0);
    }
    assert!(travel.discarded());
}

#[test]
fn a_drawn_card_flips_face_up_after_leaving_the_deck() {
    let mut travel = ready(Vec2::new(500.0, 0.0));
    travel.start = Start::Deck;
    travel.delay = 0.1;
    travel.step(0.05);
    assert!(travel.back_shown());
    assert_eq!(travel.flip_width(), 1.0);
    // The delay runs out, then the flip starts at full width and closes.
    travel.step(0.06);
    travel.step(0.06);
    assert_eq!(travel.flip_width(), 1.0);
    travel.step(0.06);
    assert!(travel.back_shown());
    assert!(travel.flip_width() < 1.0);
    travel.step(0.1);
    assert!(!travel.back_shown());
    for _ in 0..10 {
        travel.step(0.05);
    }
    assert_eq!(travel.flip_width(), 1.0);
    assert!(!travel.back_shown());
}

#[test]
fn discard_targets_the_pile_once() {
    let mut travel = ready(Vec2::ZERO);
    travel.discard = Vec2::new(900.0, -DISCARD_RISE);
    travel.discard(0.3);
    travel.step(0.1);
    travel.discard(0.3);
    assert!((travel.delay - 0.2).abs() < 1e-6);
}

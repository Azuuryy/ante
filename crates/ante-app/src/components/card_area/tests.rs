use super::*;

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-4
}

#[test]
fn the_play_area_steps_a_fixed_card_and_a_quarter_over_five_slots() {
    // play_W = 5.3 card widths, card_limit = 5: centres 1.075 apart whatever the count.
    let play = CardArea::new(AreaKind::Play, 5.3, 1.0, 5);
    for count in 1..=5 {
        let step = play.slot(1, count, 0.95).x - play.slot(0, count, 0.95).x;
        assert!(count == 1 || close(step, 1.075), "{count}: {step}");
    }
    // Two cards stay centred: the row starts 1.6125 in, minus half the size difference.
    assert!(close(play.slot(0, 2, 0.95).x, 1.6125 + 0.025));
}

#[test]
fn a_title_row_fans_like_the_hand_and_rises_by_a_fifth() {
    // view_deck: 6.5 card widths, 13 cards 0.7 wide, card_limit = 13.
    let row = CardArea::new(AreaKind::Title, 6.5, 0.7, 13);
    let first = row.slot(0, 13, 0.7);
    let middle = row.slot(6, 13, 0.7);
    let last = row.slot(12, 13, 0.7);
    assert!(close(first.x, 0.0) && close(last.x, 5.8));
    assert!(close(middle.rotation, 0.0));
    assert!(close(first.rotation, 0.2 * (-6.5 - 0.5 + 1.0) / 13.0));
    assert!(close(
        first.drop,
        (0.5_f32 * (-6.5 + 0.5) / 13.0).abs() - 0.2
    ));
    assert!(first.drop > middle.drop && close(first.drop, last.drop));
}

#[test]
fn the_hand_fan_matches_align_cards() {
    let hand = CardArea::new(AreaKind::Hand, 6.0, 1.0, 8);
    let first = hand.slot(0, 8, 1.0);
    assert!(close(first.rotation, -0.0875));
    assert!(close(first.drop, 0.21875));
}

#[test]
fn jokers_spread_fully_beyond_two_and_centre_one() {
    let jokers = CardArea::new(AreaKind::Joker, 4.9, 1.0, 5);
    assert!(close(jokers.slot(0, 1, 1.0).x, 1.95));
    // Two Jokers sit at a quarter and three quarters of the free width.
    assert!(close(jokers.slot(0, 2, 1.0).x, 3.9 * 0.25));
    assert!(close(jokers.slot(1, 2, 1.0).x, 3.9 * 0.75));
    assert!(close(jokers.slot(2, 3, 1.0).x, 3.9));
}

#[test]
fn voucher_pairs_overlap_from_alternate_sides() {
    let vouchers = CardArea::new(AreaKind::Voucher, 3.2, 1.0, 2);
    let base = vouchers.slot(0, 2, 1.0);
    let upgrade = vouchers.slot(1, 2, 1.0);
    assert!(close(base.x, 0.27));
    assert!(close(upgrade.x, 2.2 - 0.27));
    assert!(base.rotation < 0.0 && upgrade.rotation > 0.0);
}

#[test]
fn existing_helpers_agree_with_the_card_area() {
    use crate::components::tiles::{TILE, spread_x};
    // Offers and inventories place cards with the shared spread.
    let area = 600.0;
    let card = 100.0;
    let play = CardArea::new(AreaKind::Play, area, card, 5);
    for count in 1..=6 {
        for index in 0..count {
            assert!(close(
                spread_x(index, count, 5, area, card),
                play.slot(index, count, card).x
            ));
        }
    }
    // The hand fan of the hover motion.
    let hand = CardArea::new(AreaKind::Hand, 6.0, 1.0, 8);
    for count in 1..=8 {
        for index in 0..count {
            let fan = crate::components::widgets::Fan::hand(count, index);
            let slot = hand.slot(index, count, 1.0);
            assert!(close(fan.rotation(), slot.rotation));
            assert!(close(fan.drop(), slot.drop * TILE));
        }
    }
}

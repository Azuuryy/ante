use super::*;

#[test]
fn the_top_of_a_full_deck_leans_up_and_right() {
    // 51 cards under the top one: 1.26 x 0.15 x 51/52 right, 1.5 x 0.15 x 51/52 up.
    let top = deck_offset(52, 1);
    assert!((DECK_PARALLAX_X - 1.256).abs() < 0.01);
    assert!((top.x - (0.1848 * DECK_STACK_SCALE + DECK_PARALLAX_X / 30.0)).abs() < 0.002);
    assert!((top.y + 0.2207 * DECK_STACK_SCALE).abs() < 0.001);
    assert_eq!(deck_offset(52, 52).y, 0.0);
    let drawn: Vec<_> = drawn_backs(20).collect();
    assert_eq!(drawn, [20, 18, 9, 1]);
}

#[test]
fn mult_text_follows_number_format() {
    assert_eq!(mult_text(7.0), "7");
    assert_eq!(mult_text(10.5), "10.5");
    assert_eq!(mult_text(4.5), "4.50");
    assert_eq!(mult_text(123.4), "123");
    assert_eq!(mult_text(1234.0), "1,234");
}

#[test]
fn run_info_prints_each_blind_state_in_its_colour() {
    let blind = |selectable, passed, skipped| ante_core::contract::BlindView {
        id: "bl_small",
        name: "Small Blind".into(),
        target: 300,
        reward: 3,
        description: String::new(),
        selectable,
        passed,
        skipped,
        tag: None,
        can_skip: false,
    };
    use crate::components::tiles::GREY;
    assert_eq!(
        run_info_state(&blind(false, false, false), true),
        ("Current", RED)
    );
    assert_eq!(
        run_info_state(&blind(false, true, false), false),
        ("Defeated", GREY)
    );
    assert_eq!(
        run_info_state(&blind(false, false, true), false),
        ("Skipped", BLUE)
    );
    assert_eq!(
        run_info_state(&blind(true, false, false), false),
        ("Select", GOLD)
    );
    assert_eq!(
        run_info_state(&blind(false, false, false), false),
        ("Upcoming", ORANGE)
    );
}

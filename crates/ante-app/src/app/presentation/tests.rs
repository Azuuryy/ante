use super::*;

#[test]
fn font_sizes_round_to_whole_physical_pixels_and_keep_their_size() {
    for scale in [1.0, 1.12, 1.17, 1.5] {
        let size = snap_font_size(23.36, scale);
        assert!(((size * scale) - (size * scale).round()).abs() < 1e-3);
        assert!((size - 23.36).abs() <= 0.5 / scale + 1e-3);
    }
}

#[test]
fn popups_read_like_the_original_status_text() {
    assert_eq!(multiplier(150), "1.5");
    assert_eq!(multiplier(200), "2");
    assert_eq!(multiplier(125), "1.25");
    assert_eq!(popup_text(Popup::XMult(150)).0, "X1.5 Mult");
    assert_eq!(popup_text(Popup::Mult(4)).0, "+4 Mult");
    assert_eq!(popup_text(Popup::Chips(11)).0, "+11");
    assert_eq!(popup_text(Popup::Money(3)).0, "$3");
    assert_eq!(popup_text(Popup::Money(-2)).0, "-$2");
}

#[test]
fn glyphs_start_on_a_whole_pixel_and_keep_their_size() {
    // An odd-sized glyph keeps its half-pixel centre; its corner lands on a pixel.
    let snapped = snap_glyph(Vec2::new(10.3, 7.8), Vec2::new(5.0, 8.0));
    assert_eq!(snapped, Vec2::new(10.5, 8.0));
    assert_eq!(snapped - Vec2::new(2.5, 4.0), Vec2::new(8.0, 4.0));
    let aligned = Vec2::new(12.5, 4.0);
    assert_eq!(snap_glyph(aligned, Vec2::new(5.0, 8.0)), aligned);
}

#[test]
fn a_popup_square_grows_like_one_backdrop_particle() {
    assert_eq!(PopupBackdrop::side(0.0), 0.0);
    // 2 × age / 5 × 2.4: 0.96 units a second, capped at 2.4.
    assert!((PopupBackdrop::side(1.0) - 0.96).abs() < 1e-5);
    assert!((PopupBackdrop::side(2.5) - 2.4).abs() < 1e-5);
    assert!(
        PopupBackdrop::side(4.9) < 0.2,
        "it shrinks at the end of its life"
    );
}

#[test]
fn chips_and_money_popups_are_larger_than_mult_ones() {
    assert_eq!(popup_scale(Popup::Chips(30)), 1.0);
    assert_eq!(popup_scale(Popup::Money(3)), 1.0);
    assert_eq!(popup_scale(Popup::XMult(200)), 0.7);
    assert_eq!(popup_scale(Popup::Again), 0.7);
}

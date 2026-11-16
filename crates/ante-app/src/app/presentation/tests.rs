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

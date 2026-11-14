use super::*;
use ante_core::contract::PackFamily::{self, *};

const FAMILIES: [PackFamily; 5] = [Arcana, Celestial, Spectral, Standard, Buffoon];

#[test]
fn pack_cards_share_one_row_inside_the_area() {
    for family in FAMILIES {
        for count in 1..=5 {
            let area = pack_area_width(Some(family), count);
            let lefts: Vec<f32> = (0..count).map(|k| pack_card_left(k, count, area)).collect();
            assert!(lefts[0] >= 0.0, "{family:?} × {count}");
            assert!(
                lefts.windows(2).all(|pair| pair[1] > pair[0]),
                "{family:?} × {count}"
            );
            let right = lefts[count - 1] + menu_ui::CARD_WIDTH;
            assert!(
                right <= area + 1e-3,
                "{family:?} × {count}: {right} > {area}"
            );
        }
    }
}

#[test]
fn pack_areas_follow_the_original_widths() {
    let card = menu_ui::CARD_W * menu_ui::TILE;
    assert!((pack_area_width(Some(Arcana), 3) - 3.0 * card).abs() < 1e-3);
    assert!((pack_area_width(Some(Standard), 5) - 5.5 * card).abs() < 1e-3);
    let celestial = 5.5 * card + 0.5 * menu_ui::TILE;
    assert!((pack_area_width(Some(Celestial), 5) - celestial).abs() < 1e-3);
    // A five-card Standard pack steps 1.125 cards: drawn cards keep a gap.
    let area = pack_area_width(Some(Standard), 5);
    let step = pack_card_left(1, 5, area) - pack_card_left(0, 5, area);
    assert!((step - 1.125 * card).abs() < 1e-3);
}

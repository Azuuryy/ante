use super::*;

#[test]
fn deck_previews_use_distinct_in_bounds_atlas_cells() {
    use crate::app::navigation::DeckChoice;
    let atlases = Atlases {
        card_surfaces: Some(Handle::default()),
        ..default()
    };
    let mut rects = Vec::new();
    for choice in DeckChoice::ALL {
        let rect = atlases.deck_back(choice).unwrap().rect.unwrap();
        assert!(rect.min.x >= 0.0 && rect.min.y >= 0.0);
        assert!(rect.max.x <= 497.0 && rect.max.y <= 475.0);
        assert!(!rects.contains(&rect));
        rects.push(rect);
    }
}

#[test]
#[ignore = "requires an explicitly provided local installation"]
fn decode_local_card_and_joker_atlases() {
    let root = PathBuf::from(std::env::var_os("ANTE_TEST_ASSET_DIR").unwrap());
    assert!(read_atlas(&root, "Vouchers.png", (639, 380)).is_ok());
    let deck = read_atlas(&root, "8BitDeck.png", (923, 380)).unwrap();
    let jokers = read_atlas(&root, "Jokers.png", (710, 1520)).unwrap();
    assert_eq!(deck.width(), 923);
    assert_eq!(jokers.height(), 1520);
    let consumables = read_atlas(&root, "Tarots.png", (710, 570)).unwrap();
    for (file, size) in [
        ("BlindChips.png", (714, 1054)),
        ("tags.png", (204, 170)),
        ("chips.png", (145, 58)),
    ] {
        assert!(read_atlas(&root, file, size).is_ok(), "{file}");
    }
    assert_eq!((consumables.width(), consumables.height()), (710, 570));
    let surfaces = read_atlas(&root, "Enhancers.png", (497, 475)).unwrap();
    assert_eq!((surfaces.width(), surfaces.height()), (497, 475));
    let logo = read_atlas(&root, "balatro.png", (333, 216)).unwrap();
    assert_eq!((logo.width(), logo.height()), (333, 216));
    let ui_assets = read_atlas(&root, "ui_assets.png", (72, 36)).unwrap();
    assert_eq!((ui_assets.width(), ui_assets.height()), (72, 36));
    assert!(read_local_font(&root).is_ok());
}

#[test]
fn menu_swirl_requires_the_sheet_and_uses_cell_2_0() {
    assert!(Atlases::default().menu_swirl().is_none());
    let atlases = Atlases {
        ui_assets: Some(Handle::default()),
        ..default()
    };
    let swirl = atlases.menu_swirl().unwrap();
    assert_eq!(swirl.rect, Some(Rect::new(36.0, 0.0, 54.0, 18.0)));
}

#[test]
fn rejects_missing_and_invalid_assets() {
    let missing = std::env::temp_dir().join("ante-nonexistent-asset-source");
    assert!(read_atlas(&missing, "balatro.png", (333, 216)).is_err());
    assert!(read_local_font(&missing).is_err());
    assert!(validate_font(b"invalid font").is_err());
}

#[test]
fn rejects_invalid_atlas_before_decode() {
    assert!(validate_png(b"not a PNG", 923, 380).is_err());
    let mut header = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
    header.extend_from_slice(&u32::MAX.to_be_bytes());
    header.extend_from_slice(&380_u32.to_be_bytes());
    assert!(validate_png(&header, 923, 380).is_err());
}

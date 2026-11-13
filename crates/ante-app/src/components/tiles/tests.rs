use super::*;

#[test]
fn cards_spread_like_the_original_areas() {
    // Two Jokers in five slots sit centred where slots 2 and 3 of five would be.
    let (area, card) = (500.0, 100.0);
    assert!((spread_x(0, 2, 5, area, card) - 150.0).abs() < 1e-3);
    assert!((spread_x(1, 2, 5, area, card) - 250.0).abs() < 1e-3);
    // A full row spans the area; overflow overlaps.
    assert!((spread_x(4, 5, 5, area, card) - 400.0).abs() < 1e-3);
    assert!((spread_x(5, 6, 5, area, card) - 400.0).abs() < 1e-3);
    assert!((spread_x(0, 1, 1, area, card) - 200.0).abs() < 1e-3);
}

#[test]
fn viewport_is_covered_without_distortion() {
    for window in [CANVAS, Vec2::new(1080.0, 700.0), Vec2::new(3440.0, 1440.0)] {
        let (canvas, scale) = viewport_fit(window);
        assert!((canvas * scale - window).length() < 1e-3);
        assert!((canvas + Vec2::splat(1e-3)).cmpge(CANVAS).all());
    }
}

#[test]
fn the_room_unit_matches_the_original_at_1080p() {
    // love.resize: 1056 px of height hold 12.9 tiles, about 81.9 px per tile.
    let (_, scale) = viewport_fit(Vec2::new(1920.0, 1056.0));
    assert!((TILE * scale - 1056.0 / 12.9).abs() < 0.1);
    let card = card_px();
    assert!((card.x - 142.1).abs() < 0.1 && (card.y - 190.8).abs() < 0.1);
}

#[test]
fn fit_of_zero_window_is_finite() {
    let (canvas, scale) = viewport_fit(Vec2::ZERO);
    assert_eq!(canvas, CANVAS);
    assert_eq!(scale, 1.0);
}

#[test]
fn tile_and_text_helpers() {
    assert_eq!(tiles(1.0), Val::Px(73.0));
    assert_eq!(tile_px(2.0), 146.0);
    // Text at scale 1 has an em of one room unit.
    assert!((text_px(0.9) - 65.7).abs() < 1e-3);
}

#[test]
fn pointer_feedback_changes_only_button_color() {
    use bevy::{
        camera::NormalizedRenderTarget,
        picking::{
            backend::HitData,
            events::Pointer,
            pointer::{Location, PointerId},
        },
    };

    let mut world = World::new();
    world.init_resource::<Messages<Click>>();
    let style = Node {
        width: px(100),
        height: px(50),
        ..default()
    };
    let mut commands = world.commands();
    let mut button = commands.spawn((style.clone(), BackgroundColor(RED)));
    attach_button(&mut button, RED, Click::Quit);
    let entity = button.id();
    world.flush();

    let pointer = Pointer::new(
        PointerId::Mouse,
        Location {
            target: NormalizedRenderTarget::None {
                width: 100,
                height: 50,
            },
            position: Vec2::ZERO,
        },
    );
    let hit = HitData::new(Entity::PLACEHOLDER, 0.0, None, None);
    world.trigger(PointerOver {
        entity,
        pointer: pointer.clone(),
        hit: hit.clone(),
    });
    assert_eq!(
        world.get::<BackgroundColor>(entity).unwrap().0,
        hover_overlay(RED)
    );
    world.trigger(PointerPress {
        entity,
        pointer: pointer.clone(),
        button: PointerButton::Primary,
        hit: hit.clone(),
        count: 1,
    });
    assert_eq!(
        world.get::<BackgroundColor>(entity).unwrap().0,
        scale_rgb(RED, 0.52)
    );
    assert_eq!(world.get::<Node>(entity), Some(&style));
    assert_eq!(
        world.get::<UiTransform>(entity),
        Some(&UiTransform::IDENTITY)
    );
    world.trigger(PointerOut {
        entity,
        pointer,
        hit,
    });
    assert_eq!(world.get::<BackgroundColor>(entity).unwrap().0, RED);
}

#[test]
fn colour_helpers_multiply_channels() {
    let base = Color::srgb(1.0, 0.5, 0.0);
    let rim = emboss(base).to_srgba();
    assert!((rim.red - 0.7).abs() < 1e-6);
    assert!((rim.green - 0.35).abs() < 1e-6);
    let hover = hover_overlay(base).to_srgba();
    let factor = 1.0 - 0x55 as f32 / 255.0;
    assert!((hover.red - factor).abs() < 1e-6);
    assert!((hover.alpha - 1.0).abs() < 1e-6);
}

use super::*;

#[test]
fn a_restored_pose_continues_without_a_jump() {
    let fan = Fan::hand(8, 0);
    let mut before = HoverMotion { fan, ..default() };
    for _ in 0..120 {
        before.step(1.0 / 60.0, 0.0);
    }
    let pose = before.pose();
    assert!(pose.tilt < -0.05 && pose.drop > 10.0);
    let mut rebuilt = HoverMotion { fan, ..default() };
    rebuilt.restore(pose);
    rebuilt.step(1.0 / 60.0, 1.0 / 60.0);
    assert!((rebuilt.pose().tilt - pose.tilt).abs() < 0.01);
    assert!((rebuilt.pose().drop - pose.drop).abs() < 1.0);
}

#[test]
fn hand_fan_is_symmetric_and_drops_toward_the_edges() {
    let fan: Vec<Fan> = (0..8).map(|k| Fan::hand(8, k)).collect();
    assert!((fan[0].rotation + fan[7].rotation).abs() < 1e-6);
    assert!(fan[0].rotation < 0.0 && fan[7].rotation > 0.0);
    assert!((fan[0].rotation + 0.0875).abs() < 1e-6);
    assert!(fan[0].drop > fan[3].drop && fan[7].drop > fan[4].drop);
    assert_eq!(Fan::hand(1, 0).rotation, 0.0);
}

#[test]
fn hover_eases_to_the_original_scale_and_settles() {
    let mut motion = HoverMotion {
        hovered: true,
        ..default()
    };
    let (first, _, _) = motion.step(1.0 / 60.0, 0.0);
    assert!(first > 1.0 && first < HOVER_SCALE);
    for _ in 0..60 {
        motion.step(1.0 / 60.0, 0.0);
    }
    assert_eq!(motion.step(1.0 / 60.0, 0.0), (HOVER_SCALE, 0.0, 0.0));
}

#[test]
fn raise_eases_toward_the_highlight_lift() {
    let mut motion = HoverMotion {
        raise: HIGHLIGHT_LIFT,
        ..default()
    };
    let (_, lift, _) = motion.step(1.0 / 60.0, 0.0);
    assert!(lift > 0.0 && lift < HIGHLIGHT_LIFT);
    for _ in 0..60 {
        motion.step(1.0 / 60.0, 0.0);
    }
    assert_eq!(motion.step(1.0 / 60.0, 0.0).1, HIGHLIGHT_LIFT);
}

#[test]
fn juice_wobbles_then_stops_after_its_duration() {
    let mut motion = HoverMotion::default();
    motion.juice_up(0.6, 0.1, true);
    let (scale, _, rotation) = motion.step(0.01, 0.0);
    assert!(scale != 1.0 && rotation != 0.0);
    motion.step(JUICE_TIME, 0.0);
    assert_eq!(motion.step(0.01, 0.0), (1.0, 0.0, 0.0));
}

#[test]
fn card_areas_use_original_proportions() {
    assert!((PLAY_AREA_RISE - 262.8).abs() < 1e-3);
    assert!((HIGHLIGHT_LIFT - 38.16).abs() < 0.01);
}

#[test]
fn played_cards_keep_the_play_area_step_apart() {
    // `play_W = 5.3 × CARD_W` over `card_limit = 5`: centres 1.075 cards apart.
    let card = menu_ui::CARD_W * menu_ui::TILE;
    for count in 1..=5 {
        let step = CARD_WIDTH + row_gap(count, true);
        assert!((step - 1.075 * card).abs() < 1e-3, "{count} played cards");
        assert!(row_gap(count, true) > 0.0);
    }
}

#[test]
fn hand_rows_keep_their_spacing() {
    let slots = 8.0 - 1.0;
    let expected = (HAND_AREA - CARD_WIDTH) / slots - CARD_WIDTH;
    assert!((row_gap(5, false) - expected).abs() < 1e-3);
}

#[test]
fn a_drag_shift_eases_toward_its_slot() {
    let mut motion = HoverMotion {
        shift_target: 120.0,
        ..default()
    };
    motion.step(1.0 / 60.0, 0.0);
    assert!(motion.shift > 0.0 && motion.shift < 120.0);
    for _ in 0..60 {
        motion.step(1.0 / 60.0, 0.0);
    }
    assert_eq!(motion.shift, 120.0);
}

/// `ease_hover` rewrites every resting card's transform each frame; a drag shift must
/// survive it, or neighbours only move once the drop rebuilds the row.
#[test]
fn ease_hover_keeps_a_neighbours_drag_shift() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<crate::services::assets::sounds::Sfx>()
        .add_systems(Update, ease_hover);
    let card = app
        .world_mut()
        .spawn((
            Node::default(),
            UiTransform::IDENTITY,
            Visibility::Inherited,
            HoverMotion {
                shift_target: 80.0,
                ..default()
            },
        ))
        .id();
    for _ in 0..30 {
        app.update();
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    let x = match app.world().get::<UiTransform>(card).unwrap().translation.x {
        Val::Px(x) => x,
        other => panic!("{other:?}"),
    };
    assert!(x > 1.0, "translation {x}");
}

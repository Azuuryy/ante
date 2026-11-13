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

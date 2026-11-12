//! The original room sway and jiggle (`update_canvas_juice`): the whole board drifts
//! slowly, leans a little toward the cursor and shakes when score popups land.

use bevy::{math::Affine2, prelude::*, ui::UiGlobalTransform, window::PrimaryWindow};

use crate::{app::presentation::Screen, components::tiles::TILE};

/// Original default `screenshake` of 50 gives a shake amount of 50 / 100 × 3.
const SHAKE: f32 = 1.5;
/// The cursor pull uses `max(0, screenshake − 30) / 100`.
const CURSOR_PULL: f32 = 0.2;

/// Jiggle added by popups and decayed by `(1 − 5·dt)` each frame, like `G.ROOM.jiggle`.
#[derive(Resource, Default)]
pub(crate) struct RoomShake {
    pub(crate) jiggle: f32,
    cursor: Option<Vec2>,
}

impl RoomShake {
    /// Each scoring popup adds 0.7, like `card_eval_status_text`.
    pub(crate) fn kick(&mut self, amount: f32) {
        self.jiggle += amount;
    }

    /// Room offset in room units and rotation in radians at real time `t`.
    fn sway(&self, t: f32, cursor: Vec2) -> (Vec2, f32) {
        let j = self.jiggle;
        // The original also leans 0.001 × sin(0.3 t) at rest; with crisp pixel edges that
        // reads as a crawling step, so the room only turns while it shakes.
        let rotation = 0.002 * j * (39.913 * t).sin() * SHAKE;
        let x = SHAKE
            * (0.015 * (0.913 * t).sin() + 0.01 * j * SHAKE * (19.913 * t).sin() + cursor.x * 0.01);
        let y = SHAKE
            * (0.015 * (0.952 * t).sin() + 0.01 * j * SHAKE * (21.913 * t).sin() + cursor.y * 0.01);
        (Vec2::new(x, y), rotation)
    }
}

pub(crate) fn shake_room(
    time: Res<Time<Real>>,
    mut shake: ResMut<RoomShake>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut roots: Query<&mut UiTransform, With<Screen>>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();
    let shake = shake.as_mut();
    shake.jiggle *= (1.0 - 5.0 * dt).max(0.0);
    if shake.jiggle < 0.001 {
        shake.jiggle = 0.0;
    }
    // Cursor offset from the room centre in room units, eased at 3·dt.
    let target = windows
        .single()
        .ok()
        .and_then(|window| {
            let size = window.size();
            window
                .cursor_position()
                .map(|cursor| CURSOR_PULL * (cursor - size / 2.0) / (size.y / 11.5))
        })
        .unwrap_or(Vec2::ZERO);
    let eased = shake.cursor.get_or_insert(target);
    *eased += (target - *eased) * (3.0 * dt).min(1.0);
    let eased = *eased;
    let (offset, rotation) = shake.sway(t, eased);
    for mut transform in &mut roots {
        transform.translation = Val2::new(Val::Px(offset.x * TILE), Val::Px(offset.y * TILE));
        transform.rotation = Rot2::radians(rotation);
    }
}

/// Screen-space transform of the room sway, so card positions can be kept in room space
/// and the sway never reads as card movement.
pub(crate) fn room_frame(
    root: Option<(&UiGlobalTransform, &UiTransform, &ComputedNode)>,
) -> Affine2 {
    let Some((global, transform, node)) = root else {
        return Affine2::IDENTITY;
    };
    let moved = Vec2::new(
        px_value(transform.translation.x),
        px_value(transform.translation.y),
    ) / node.inverse_scale_factor();
    let centre = global.affine().translation - moved;
    global.affine() * Affine2::from_translation(-centre)
}

fn px_value(value: Val) -> f32 {
    match value {
        Val::Px(value) => value,
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jiggle_shakes_and_rest_only_drifts() {
        let rest = RoomShake::default();
        let (offset, rotation) = rest.sway(1.0, Vec2::ZERO);
        assert!(offset.length() < 0.05 && rotation == 0.0);
        let shaken = RoomShake {
            jiggle: 3.0,
            cursor: None,
        };
        let samples = (0..50).map(|i| shaken.sway(i as f32 * 0.013, Vec2::ZERO).1.abs());
        assert!(samples.fold(0.0, f32::max) > 0.005);
    }
}

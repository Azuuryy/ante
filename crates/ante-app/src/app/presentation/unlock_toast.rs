//! "Unlocked!" notices, like the original `unlock_notify`: each item unlocked shows its
//! name in a box at the top of the window for a few seconds, one after the other.

use super::*;
use crate::components::theme::INK;

/// Seconds each notice stays.
pub(crate) const TOAST_TIME: f32 = 3.0;

/// A notice on screen and the time it has left.
#[derive(Component)]
pub(crate) struct UnlockToast {
    left: f32,
}

/// The notice for one item.
pub(crate) fn unlock_toast(name: &str) -> impl Bundle {
    (
        UnlockToast { left: TOAST_TIME },
        Node {
            position_type: PositionType::Absolute,
            top: px(24),
            left: percent(50),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(20), px(10)),
            row_gap: px(4),
            ..default()
        },
        UiTransform::from_translation(Val2::percent(-50.0, 0.0)),
        BackgroundColor(Color::srgba(0.12, 0.18, 0.2, 0.95)),
        GlobalZIndex(2000),
        Pickable::IGNORE,
        children![
            (
                Text::new("Unlocked!"),
                TextFont {
                    font_size: bevy::text::FontSize::Px(16.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.6, 0.0)),
                Pickable::IGNORE,
            ),
            (
                Text::new(name.to_owned()),
                TextFont {
                    font_size: bevy::text::FontSize::Px(24.0),
                    ..default()
                },
                TextColor(INK),
                Pickable::IGNORE,
            )
        ],
    )
}

/// Shows the waiting notices one at a time and removes each when its time is up.
pub(crate) fn show_unlock_toasts(
    mut commands: Commands,
    mut session: ResMut<Session>,
    time: Res<Time<Real>>,
    mut shown: Query<(Entity, &mut UnlockToast)>,
) {
    let dt = time.delta_secs();
    let mut showing = false;
    for (entity, mut toast) in &mut shown {
        toast.left -= dt;
        if toast.left <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            showing = true;
        }
    }
    if showing || session.unlock_toasts.is_empty() {
        return;
    }
    if let Some(name) = session.bypass_change_detection().unlock_toasts.pop_front() {
        commands.spawn(unlock_toast(&name));
    }
}

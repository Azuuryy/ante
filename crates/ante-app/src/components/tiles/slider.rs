//! `create_slider` and `G.FUNCS.slider`: a black track filled in red up to the value, a red
//! box with the value beside it, and the label above. Dragging sets the value from the
//! pointer's position along the track.

use super::*;
use crate::services::profile::{Profile, VolumeChannel};
use bevy::window::PrimaryWindow;

/// A volume slider track, with the fill and value text it updates while dragged.
#[derive(Component, Clone, Copy)]
pub(crate) struct Slider {
    channel: VolumeChannel,
    fill: Entity,
    text: Entity,
}

/// The slider the pointer is dragging, until the button is released.
#[derive(Resource, Default)]
pub(crate) struct SliderGrab(Option<Entity>);

/// Value from 0 to 100 for a pointer at `x`, on a track from `left` that is `width` wide,
/// as `G.FUNCS.slider` computes it.
pub(crate) fn slider_value(x: f32, left: f32, width: f32) -> u8 {
    if width <= 0.0 {
        return 0;
    }
    (100.0 * (x - left) / width).clamp(0.0, 100.0).round() as u8
}

/// `create_slider({label, w = 5, h = 0.4})` bound to a volume.
pub(crate) fn volume_slider(
    ui: &mut ChildSpawnerCommands,
    label_text: &str,
    channel: VolumeChannel,
    value: u8,
) {
    let (w, h) = (5.0, 0.4);
    ui.spawn(Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        min_height: tiles(1.0),
        padding: UiRect::all(tiles(0.05)),
        ..default()
    })
    .with_children(|column| {
        label(column, label_text, text_px(0.5), INK);
        column
            .spawn(Node {
                padding: UiRect::all(tiles(0.1)),
                column_gap: tiles(0.1),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|line| {
                let mut track = line.spawn((
                    Node {
                        width: tiles(w),
                        min_height: tiles(h),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    PixelBox::new(BLACK).emboss(0.05),
                ));
                let fill = track
                    .commands()
                    .spawn((
                        Node {
                            width: percent(value),
                            height: tiles(h),
                            ..default()
                        },
                        PixelBox::new(RED),
                        Pickable::IGNORE,
                    ))
                    .id();
                track.add_child(fill);
                let track = track.id();
                let mut text = None;
                line.spawn((
                    Node {
                        min_width: tiles(0.8),
                        min_height: tiles(h),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    PixelBox::new(RED).shadow(),
                    Pickable::IGNORE,
                ))
                .with_children(|value_box| {
                    text = Some(
                        value_box
                            .spawn((
                                Text::new(value.to_string()),
                                TextFont {
                                    font_size: FontSize::Px(text_px(0.3)),
                                    ..default()
                                },
                                TextColor(INK),
                                Pickable::IGNORE,
                            ))
                            .id(),
                    );
                });
                line.commands()
                    .entity(track)
                    .insert(Slider {
                        channel,
                        fill,
                        text: text.expect("value text"),
                    })
                    .observe(|event: On<PointerPress>, mut grab: ResMut<SliderGrab>| {
                        if event.button == PointerButton::Primary {
                            grab.0 = Some(event.entity);
                        }
                    });
            });
    });
}

/// Follows the pointer while a slider is held, and saves the volumes once it is released.
pub(crate) fn drag_sliders(
    mut grab: ResMut<SliderGrab>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    sliders: Query<(&Slider, &ComputedNode, &UiGlobalTransform)>,
    mut nodes: Query<&mut Node>,
    mut texts: Query<&mut Text>,
    mut profile: ResMut<Profile>,
) {
    let Some(entity) = grab.0 else {
        return;
    };
    let Ok((slider, node, transform)) = sliders.get(entity) else {
        grab.0 = None;
        return;
    };
    if let Some(cursor) = windows
        .single()
        .ok()
        .and_then(Window::physical_cursor_position)
    {
        let width = node.size().x;
        let value = slider_value(cursor.x, transform.translation.x - width / 2.0, width);
        if profile.volume_of(slider.channel) != value {
            profile.set_volume(slider.channel, value);
            if let Ok(mut fill) = nodes.get_mut(slider.fill) {
                fill.width = percent(value);
            }
            if let Ok(mut text) = texts.get_mut(slider.text) {
                text.0 = value.to_string();
            }
        }
    }
    if !mouse.pressed(MouseButton::Left) {
        grab.0 = None;
        if let Err(_error) = profile.save_volumes() {
            crate::journal_event!(
                "profile_write_failed",
                serde_json::json!({"error": _error.to_string()})
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_value_follows_the_pointer_along_the_track() {
        assert_eq!(slider_value(100.0, 100.0, 200.0), 0);
        assert_eq!(slider_value(200.0, 100.0, 200.0), 50);
        assert_eq!(slider_value(300.0, 100.0, 200.0), 100);
        assert_eq!(slider_value(20.0, 100.0, 200.0), 0);
        assert_eq!(slider_value(900.0, 100.0, 200.0), 100);
        assert_eq!(slider_value(10.0, 0.0, 0.0), 0);
    }
}

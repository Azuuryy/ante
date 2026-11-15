//! A card area drawn as one row: each card rests where `CardArea:align_cards` puts it, with
//! the fan tilt, drop and sway of its slot, and grows when hovered like a hand card. The
//! deck view and the collection lay their cards out with it.

use bevy::{ecs::system::EntityCommands, prelude::*};

use super::{
    card_area::CardArea,
    tiles::TILE,
    widgets::{Fan, HoverMotion, card_hover},
};

/// Spawns `area` as a row `height` tiles tall holding `count` cards of `card` px; `card_at`
/// receives each card entity, already placed, to add its art, tooltip or click.
pub(crate) fn card_row(
    ui: &mut ChildSpawnerCommands,
    area: CardArea,
    height: f32,
    card: Vec2,
    count: usize,
    mut card_at: impl FnMut(&mut EntityCommands, usize),
) {
    ui.spawn((
        Node {
            width: px(area.width * TILE),
            height: px(height * TILE),
            flex_shrink: 0.0,
            ..default()
        },
        Pickable::IGNORE,
    ))
    .with_children(|row| {
        for index in 0..count {
            let slot = area.slot(index, count, card.x / TILE);
            let mut entity = row.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(slot.x * TILE),
                    top: px((height * TILE - card.y) / 2.0),
                    width: px(card.x),
                    height: px(card.y),
                    ..default()
                },
                ZIndex(index as i32),
            ));
            card_hover(&mut entity);
            entity.insert(HoverMotion::fanned(Fan::area(slot, slot.x)));
            card_at(&mut entity, index);
        }
    });
}

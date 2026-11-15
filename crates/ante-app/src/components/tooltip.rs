use super::{
    info::Info,
    pixel_rect::PixelBox,
    theme::MUTED,
    tiles::{BLACK, TILE, darken, lighten, text_px, tiles},
    widgets::{label, text_shadow},
};
use ante_core::contract::CurrentValue;
use bevy::{
    ecs::system::EntityCommands,
    picking::prelude::{Pickable, PointerOut, PointerOver},
    prelude::*,
    window::PrimaryWindow,
};

mod ink;
mod panel;
mod tip;

use ink::*;
pub(crate) use ink::{animate_edition_badges, animate_shimmer};
use panel::*;
pub use tip::*;

/// Fixed popup width, so the text wraps the same wherever the popup sits.
const TOOLTIP_WIDTH: f32 = 4.2 * TILE;
/// Space between the panel edge and its text. Wide enough that no line sits at the limit,
/// where a sub-pixel move could change the wrapping.
const PANEL_SIDE_PADDING: f32 = 12.0;
/// Side boxes (`info_queue`) hang left of the main popup, centred on it, like `align = 'cl'`.
const SIDE_WIDTH: f32 = 3.0 * TILE;
const SIDE_GAP: f32 = 4.0;
/// `G.C.JOKER_GREY`.
const JOKER_GREY: Color = Color::srgb(
    0xBF as f32 / 255.0,
    0xC7 as f32 / 255.0,
    0xD5 as f32 / 255.0,
);
/// `G.C.SECONDARY_SET.Enhanced`.
const ENHANCED: Color = Color::srgb(
    0x83 as f32 / 255.0,
    0x89 as f32 / 255.0,
    0xDD as f32 / 255.0,
);
/// `G.C.PURPLE`.
const PURPLE: Color = Color::srgb(
    0x88 as f32 / 255.0,
    0x67 as f32 / 255.0,
    0xA5 as f32 / 255.0,
);
/// Description text scale: `0.32 × DESCSCALE` (1 for m6x11plus).
const DESC_SCALE: f32 = 0.32;
/// `info_tip_from_rows` title box: the original grey lightened by 0.15.
const SIDE_PANEL: Color = Color::srgb(
    0x77 as f32 / 255.0,
    0x88 as f32 / 255.0,
    0x8B as f32 / 255.0,
);
const TEXT_DARK: Color = Color::srgb(
    0x4F as f32 / 255.0,
    0x63 as f32 / 255.0,
    0x67 as f32 / 255.0,
);
const TEXT_INACTIVE: Color = Color::srgba(
    0x88 as f32 / 255.0,
    0x88 as f32 / 255.0,
    0x88 as f32 / 255.0,
    0x99 as f32 / 255.0,
);
const PAPER: Color = Color::WHITE;
#[cfg(feature = "dev-tools")]
const DEBUG_PANEL: Color = Color::srgba(0.86, 0.88, 0.90, 0.96);
/// Room height and card height in room units, used for the original anchor rule.
const ROOM_HEIGHT_UNITS: f32 = 11.5;
const CARD_HEIGHT_UNITS: f32 = 2.4 * 47.0 / 41.0;
const GAP: f32 = 10.0;
/// Where the popup sits relative to its source, following the original placement rule.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TipAnchor {
    /// Above the source, or below it when the source is near the top of the room.
    #[default]
    Auto,
    /// Left of the source, as for shop offers and deck views.
    Left,
}

#[derive(Resource, Default)]
pub struct TooltipState {
    active: Option<(Entity, Tip, TipAnchor)>,
}

impl TooltipState {
    pub fn dismiss(&mut self) {
        self.active = None;
    }
}

#[derive(Component)]
pub(crate) struct TooltipPanel {
    source: Entity,
    tip: Tip,
}

pub fn attach(entity: &mut EntityCommands, tip: impl Into<Tip>) {
    attach_anchored(entity, tip, TipAnchor::Auto);
}

pub fn attach_anchored(entity: &mut EntityCommands, tip: impl Into<Tip>, anchor: TipAnchor) {
    let tip = tip.into();
    entity
        .observe(
            move |event: On<PointerOver>, mut state: ResMut<TooltipState>| {
                state.active = Some((event.entity, tip.clone(), anchor));
            },
        )
        .observe(|event: On<PointerOut>, mut state: ResMut<TooltipState>| {
            if state
                .active
                .as_ref()
                .is_some_and(|(id, _, _)| *id == event.entity)
            {
                state.dismiss();
            }
        });
}

pub(crate) fn update(
    mut commands: Commands,
    state: Res<TooltipState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut panels: Query<(Entity, &TooltipPanel, &ComputedNode, &mut Node)>,
    anchors: Query<(&ComputedNode, &UiGlobalTransform), Without<TooltipPanel>>,
    drag: Option<Res<super::card_drag::CardDrag>>,
) {
    let Ok(window) = windows.single() else { return };
    let active = state.active.as_ref().filter(|(source, _, _)| {
        !drag.as_ref().is_some_and(|drag| drag.moving())
            && anchors.contains(*source)
            && window.cursor_position().is_some()
    });
    let current = panels.single_mut().ok();
    let Some((source, tip, anchor)) = active else {
        if let Some((entity, _, _, _)) = current {
            commands.entity(entity).despawn();
        }
        return;
    };
    let Ok((node, transform)) = anchors.get(*source) else {
        return;
    };
    let dpi = window.scale_factor();
    let half = node.size() / dpi / 2.0;
    let center = transform.translation / dpi;
    let width = window.width().min(TOOLTIP_WIDTH);
    let side = if tip.side.is_empty() {
        0.0
    } else {
        SIDE_WIDTH + SIDE_GAP
    };
    // The main popup follows the original anchor rule; side boxes extend to its left.
    let place = |height: f32| {
        let pos = position(
            center,
            half,
            Vec2::new(width, height),
            window.size(),
            *anchor,
        );
        // Whole physical pixels, so text wraps the same wherever the panel sits.
        Vec2::new((pos.x - side).max(0.0), pos.y)
            .mul_add(Vec2::splat(dpi), Vec2::ZERO)
            .round()
            / dpi
    };
    if let Some((entity, existing, measured, mut layout)) = current {
        if existing.source == *source && existing.tip == *tip {
            let pos = place(measured.size().y / dpi);
            layout.left = px(pos.x);
            layout.top = px(pos.y);
            return;
        }
        commands.entity(entity).despawn();
    }
    let pos = place(150.0);
    commands
        .spawn((
            TooltipPanel {
                source: *source,
                tip: tip.clone(),
            },
            Node {
                position_type: PositionType::Absolute,
                left: px(pos.x),
                top: px(pos.y),
                width: px(width + side),
                max_height: px(window.height()),
                row_gap: px(4),
                align_items: AlignItems::FlexEnd,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            GlobalZIndex(1000),
            Pickable::IGNORE,
        ))
        .with_children(|stack| {
            // Side boxes are centred on the main popup only, like `align = 'cl'`.
            stack
                .spawn((
                    Node {
                        column_gap: px(SIDE_GAP),
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|row| {
                    if !tip.side.is_empty() {
                        row.spawn((
                            Node {
                                width: px(SIDE_WIDTH),
                                flex_shrink: 0.0,
                                row_gap: px(4),
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                            Pickable::IGNORE,
                        ))
                        .with_children(|boxes| {
                            for info in &tip.side {
                                side_box(boxes, info);
                            }
                        });
                    }
                    // `card_h_popup`: a JOKER_GREY frame around a translucent dark body.
                    row.spawn((
                        Node {
                            width: px(width),
                            flex_shrink: 0.0,
                            padding: UiRect::all(tiles(0.05)),
                            ..default()
                        },
                        PixelBox::new(lighten(JOKER_GREY, 0.5)).emboss(0.07),
                        Pickable::IGNORE,
                    ))
                    .with_children(|frame| {
                        frame
                            .spawn((
                                Node {
                                    flex_grow: 1.0,
                                    padding: UiRect::axes(px(PANEL_SIDE_PADDING), tiles(0.07)),
                                    row_gap: tiles(0.07),
                                    align_items: AlignItems::Center,
                                    flex_direction: FlexDirection::Column,
                                    ..default()
                                },
                                PixelBox::new(darken(BLACK, 0.1).with_alpha(0.8)),
                                Pickable::IGNORE,
                            ))
                            .with_children(|panel| main_panel(panel, tip));
                    });
                });
            #[cfg(feature = "dev-tools")]
            if !tip.technical.is_empty() {
                stack
                    .spawn((
                        Node {
                            width: px(width),
                            padding: UiRect::axes(px(PANEL_SIDE_PADDING), px(6)),
                            border_radius: BorderRadius::all(px(6)),
                            row_gap: px(2),
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        BackgroundColor(DEBUG_PANEL),
                        Pickable::IGNORE,
                    ))
                    .with_children(|debug| {
                        for line in &tip.technical {
                            label(debug, line, 14.0, TEXT_DARK);
                        }
                    });
            }
        });
}

/// Original rule: above the card, below it when its top is within 0.8 card heights of
/// the room top, left of it for shop offers; flipped then clamped to stay on screen.
fn position(center: Vec2, half: Vec2, size: Vec2, window: Vec2, anchor: TipAnchor) -> Vec2 {
    let unit = window.y / ROOM_HEIGHT_UNITS;
    let pos = match anchor {
        TipAnchor::Left => {
            let left = center.x - half.x - GAP - size.x;
            let x = if left >= 0.0 {
                left
            } else {
                center.x + half.x + GAP
            };
            Vec2::new(x, center.y - size.y / 2.0)
        }
        TipAnchor::Auto => {
            let above = center.y - half.y - GAP - size.y;
            let below = center.y + half.y + GAP;
            let near_top = center.y - half.y < 0.8 * CARD_HEIGHT_UNITS * unit;
            let y = if (near_top || above < 0.0) && below + size.y <= window.y {
                below
            } else {
                above
            };
            Vec2::new(center.x - size.x / 2.0, y)
        }
    };
    pos.clamp(Vec2::ZERO, (window - size).max(Vec2::ZERO))
}

#[cfg(test)]
mod tests;

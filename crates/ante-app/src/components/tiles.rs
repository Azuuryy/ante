use crate::{
    app::{navigation::DeckChoice, session::Click},
    components::{
        pixel_rect::PixelBox,
        theme::{INK, MUTED},
        widgets::{atlas_image, label, shadowed_label},
    },
};
use bevy::{
    picking::prelude::{
        Pickable, PointerButton, PointerClick, PointerOut, PointerOver, PointerPress,
        PointerRelease,
    },
    prelude::*,
    text::FontSize,
};

mod buttons;
mod palette;

pub(crate) use buttons::*;
pub(crate) use palette::*;

/// Balatro draws everything in tile units: 1 tile = 20 base px x TILESCALE 3.65.
pub const TILE: f32 = 73.0;
/// Virtual room: 20 x 11.5 tiles at the default TILESCALE.
pub const ROOM: Vec2 = Vec2::new(1460.0, 839.5);
/// The original fits the room plus `ROOM_PADDING_W` = 1 and `ROOM_PADDING_H` = 0.7 tiles
/// on each side to the window (`love.resize`), so the canvas is 22 x 12.9 tiles.
pub const CANVAS: Vec2 = Vec2::new(1606.0, 941.7);
/// Card size in tiles (`G.CARD_W`, `G.CARD_H`) and the drawn scale of a card (`Card.T.scale`).
pub const CARD_W: f32 = 2.4 * 35.0 / 41.0;
pub const CARD_H: f32 = 2.4 * 47.0 / 41.0;
pub const CARD_SCALE: f32 = 0.95;

/// Drawn card size in UI pixels: 0.95 x (CARD_W, CARD_H) tiles.
pub fn card_px() -> Vec2 {
    Vec2::new(CARD_WIDTH, CARD_HEIGHT)
}
/// Joker and consumable areas of `set_screen_positions`: 4.9 and 2.3 cards wide.
pub const JOKER_AREA: f32 = 4.9 * CARD_W * TILE;
pub const CONSUMABLE_AREA: f32 = 2.3 * CARD_W * TILE;
/// Drawn card width and height in UI pixels.
pub const CARD_WIDTH: f32 = CARD_W * CARD_SCALE * TILE;
pub const CARD_HEIGHT: f32 = CARD_H * CARD_SCALE * TILE;

/// Left edge of card `k` of `n` in an area `area` px wide that holds `limit` cards, as the
/// Joker, consumable and shop areas of `CardArea:align_cards` spread them.
pub fn spread_x(k: usize, n: usize, limit: usize, area: f32, card: f32) -> f32 {
    let m = n.max(limit);
    if m <= 1 {
        return (area - card) / 2.0;
    }
    let span = (m - 1) as f32;
    (area - card) * (k as f32 / span + 0.5 * (m - n) as f32 / span)
}
/// Em size of text at scale 1: the original renders fonts at `render_scale` x `FONTSCALE`
/// = 20 x 0.1 steps per scale, one room unit.
pub const FONT_BASE: f32 = TILE;

pub fn tiles(t: f32) -> Val {
    Val::Px(t * TILE)
}

pub fn tile_px(t: f32) -> f32 {
    t * TILE
}

pub fn text_px(scale: f32) -> f32 {
    FONT_BASE * scale
}

/// Balatro emboss rim = darken(colour, 0.3).
pub fn emboss(color: Color) -> Color {
    scale_rgb(color, 0.7)
}

/// Balatro hover overlay #00000055 composited over the base colour.
pub fn hover_overlay(color: Color) -> Color {
    scale_rgb(color, 1.0 - 0x55 as f32 / 255.0)
}

pub fn viewport_fit(physical_size: Vec2) -> (Vec2, f32) {
    let scale = (physical_size.x / CANVAS.x).min(physical_size.y / CANVAS.y);
    if !scale.is_finite() || scale <= 0.0 {
        return (CANVAS, 1.0);
    }
    (physical_size / scale, scale)
}

pub fn screen_heading(ui: &mut ChildSpawnerCommands, text: &str) {
    label(ui, text, text_px(0.65), INK);
}

pub fn panel(ui: &mut ChildSpawnerCommands, content: impl FnOnce(&mut ChildSpawnerCommands)) {
    ui.spawn((
        Node {
            padding: UiRect::all(tiles(0.25)),
            border: UiRect::all(tiles(0.1)),
            border_radius: BorderRadius::all(tiles(0.18)),
            min_width: tiles(11.5),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: tiles(0.12),
            ..default()
        },
        BackgroundColor(L_BLACK),
        BorderColor::all(emboss(L_BLACK)),
    ))
    .with_children(content);
}

pub fn deck_preview_large(
    ui: &mut ChildSpawnerCommands,
    choice: DeckChoice,
    back: Option<ImageNode>,
) {
    let color = match choice {
        DeckChoice::Red => Color::srgb(0.65, 0.18, 0.20),
        DeckChoice::Blue => Color::srgb(0.15, 0.36, 0.65),
        _ => Color::srgb(0.27, 0.29, 0.29),
    };
    ui.spawn((
        Node {
            width: tiles(2.0),
            height: tiles(2.3),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(color),
    ))
    .with_children(|card| {
        if let Some(image) = back {
            atlas_image(card, image, tile_px(1.45), tile_px(1.95));
        } else {
            label(card, "DECK", text_px(0.4), INK);
        }
    });
}

#[cfg(test)]
mod tests;

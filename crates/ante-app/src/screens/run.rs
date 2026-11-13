use crate::components::{
    pixel_rect::{DynRole, PixelBox},
    tiles::{BLACK, CARD_H, CARD_W, IMPORTANT, MONEY, TILE, TRANSPARENT_DARK, darken, lighten},
    widgets::{shadowed_label, text_shadow},
};
use crate::{
    app::{
        navigation::{DeckChoice, STAKES, Screen as Page},
        playback::Speed,
        presentation::Screen,
        session::{Click, Session, parse_seed},
    },
    components::{
        cards::{inventory_cards, voucher_card},
        theme::{INK, MUTED},
        tiles::{BLUE, ORANGE, RED, text_px, tile_button, tile_optional},
        widgets::{atlas_image, card_hover, label, row, wrap_row},
    },
    screens::{
        hand::playing_board,
        shop::{pack_board, shop_board},
    },
    services::assets::catalog::Atlases,
};
use ante_core::contract::{BlindView, Command, HandKind, Phase, RANKS, RunView, SUITS};
use bevy::prelude::*;

mod blind;
mod board;
mod deck;
mod deck_select;
mod hud;
mod pause;
mod result;
mod shop_sign;
mod text;

use blind::*;
pub use board::*;
pub use deck::*;
pub use deck_select::*;
pub use hud::*;
pub use pause::*;
pub use result::*;
pub(crate) use shop_sign::*;
pub(crate) use text::*;

const PANEL: Color = Color::srgb(0.20, 0.27, 0.28);
const WELL: Color = Color::srgb(0.10, 0.15, 0.16);
const CREAM: Color = Color::srgb(0.94, 0.95, 0.91);
const OUTLINE: Color = Color::srgb(0.66, 0.76, 0.77);
const GOLD: Color = Color::srgb(0.95, 0.71, 0.30);

fn column(gap: f32) -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        row_gap: px(gap),
        ..default()
    }
}

fn box_node(node: Node, color: Color) -> (Node, BackgroundColor) {
    (
        Node {
            border_radius: BorderRadius::all(px(12)),
            ..node
        },
        BackgroundColor(color),
    )
}

pub fn overlay(ui: &mut ChildSpawnerCommands, content: impl FnOnce(&mut ChildSpawnerCommands)) {
    ui.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.31, 0.40, 0.40, 0.65)),
        GlobalZIndex(200),
        Pickable {
            should_block_lower: true,
            is_hoverable: false,
        },
    ))
    .with_children(content);
}

fn modal(
    ui: &mut ChildSpawnerCommands,
    width: f32,
    gap: f32,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    ui.spawn((
        Node {
            width: px(width),
            flex_shrink: 0.0,
            padding: UiRect::all(px(16)),
            border: UiRect::all(px(4)),
            border_radius: BorderRadius::all(px(16)),
            ..column(gap)
        },
        BackgroundColor(PANEL),
        BorderColor::all(OUTLINE),
        BoxShadow::new(
            Color::srgba(0.0, 0.0, 0.0, 0.30),
            px(0),
            px(7),
            px(0),
            px(0),
        ),
    ))
    .with_children(content);
}

#[cfg(test)]
mod tests;

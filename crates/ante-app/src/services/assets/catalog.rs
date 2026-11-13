use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

use bevy::{
    asset::RenderAssetUsages,
    image::{CompressedImageFormats, ImageSampler, ImageType},
    prelude::*,
};
#[cfg(feature = "dev-tools")]
use serde_json::json;

use crate::services::diagnostics::FrameMetrics;

mod content_art;
mod local;

pub use local::*;

const TILE_WIDTH: u32 = 71;
const TILE_HEIGHT: u32 = 95;
const MAX_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Resource)]
pub struct AtlasSource(pub Option<PathBuf>);

#[derive(Resource, Default)]
pub struct Atlases {
    cards: Option<Handle<Image>>,
    card_surfaces: Option<Handle<Image>>,
    jokers: Option<Handle<Image>>,
    vouchers: Option<Handle<Image>>,
    boosters: Option<Handle<Image>>,
    shop_sign: Option<Handle<Image>>,
    consumables: Option<Handle<Image>>,
    ui_assets: Option<Handle<Image>>,
    blind_chips: Option<Handle<Image>>,
    tags: Option<Handle<Image>>,
    stake_chips: Option<Handle<Image>>,
    pub logo: Option<Handle<Image>>,
    pub font: Option<Handle<Font>>,
    pub status: Option<&'static str>,
}

impl Atlases {
    pub fn blind_chip(&self, id: &str) -> Option<ImageNode> {
        let (x, y) = match id {
            "bl_small" => (0, 0),
            "bl_big" => (0, 1),
            "bl_ox" => (0, 2),
            "bl_hook" => (0, 7),
            "bl_mouth" => (0, 18),
            "bl_fish" => (0, 5),
            "bl_club" => (0, 4),
            "bl_manacle" => (0, 8),
            "bl_tooth" => (0, 22),
            "bl_wall" => (0, 9),
            "bl_house" => (0, 3),
            "bl_mark" => (0, 23),
            "bl_final_bell" => (0, 26),
            "bl_wheel" => (0, 10),
            "bl_arm" => (0, 11),
            "bl_psychic" => (0, 12),
            "bl_goad" => (0, 13),
            "bl_water" => (0, 14),
            "bl_eye" => (0, 17),
            "bl_plant" => (0, 19),
            "bl_needle" => (0, 20),
            "bl_head" => (0, 21),
            "bl_final_leaf" => (0, 28),
            "bl_final_vessel" => (0, 29),
            "bl_window" => (0, 6),
            "bl_serpent" => (0, 15),
            "bl_pillar" => (0, 16),
            "bl_flint" => (0, 24),
            "bl_final_acorn" => (0, 27),
            "bl_final_heart" => (0, 25),
            _ => return None,
        };
        Some(square(self.blind_chips.clone()?, 34.0, x, y))
    }

    pub fn tag_icon(&self, id: &str) -> Option<ImageNode> {
        let (x, y) = match id {
            "tag_uncommon" => (0, 0),
            "tag_rare" => (1, 0),
            "tag_negative" => (2, 0),
            "tag_foil" => (3, 0),
            "tag_holo" => (0, 1),
            "tag_polychrome" => (1, 1),
            "tag_investment" => (2, 1),
            "tag_voucher" => (3, 1),
            "tag_boss" => (0, 2),
            "tag_standard" => (1, 2),
            "tag_charm" => (2, 2),
            "tag_meteor" => (3, 2),
            "tag_buffoon" => (4, 2),
            "tag_handy" => (1, 3),
            "tag_garbage" => (2, 3),
            "tag_ethereal" => (3, 3),
            "tag_coupon" => (4, 0),
            "tag_double" => (5, 0),
            "tag_juggle" => (5, 1),
            "tag_d_six" => (5, 3),
            "tag_top_up" => (4, 1),
            "tag_skip" => (0, 3),
            "tag_orbital" => (5, 2),
            "tag_economy" => (4, 3),
            "tag_speed" => (0, 3),
            _ => return None,
        };
        Some(square(self.tags.clone()?, 34.0, x, y))
    }

    pub fn stake_chip(&self, stake: usize) -> Option<ImageNode> {
        let &(x, y) = [
            (0, 0),
            (1, 0),
            (2, 0),
            (4, 0),
            (3, 0),
            (0, 1),
            (1, 1),
            (2, 1),
        ]
        .get(stake)?;
        Some(square(self.stake_chips.clone()?, 29.0, x, y))
    }

    pub fn card(&self, rank: char, suit: &str) -> Option<ImageNode> {
        let column = "23456789TJQKA".find(rank)? as u32;
        let row = match suit {
            "Hearts" => 0,
            "Clubs" => 1,
            "Diamonds" => 2,
            "Spades" => 3,
            _ => return None,
        };
        Some(tile(self.cards.clone()?, column, row))
    }

    pub fn card_base(&self) -> Option<ImageNode> {
        Some(tile(self.card_surfaces.clone()?, 1, 0))
    }

    /// The card centre for an engine enhancement label: `Card:set_ability` swaps the plain
    /// base for the enhancement's sprite (Gold, Steel, Glass…).
    pub fn card_base_for(&self, enhancement: &str) -> Option<ImageNode> {
        enhancement_id(enhancement)
            .and_then(|id| self.enhancement(id))
            .or_else(|| self.card_base())
    }

    /// Enhanced card centres, at their original `Enhancers.png` positions.
    pub fn enhancement(&self, id: &str) -> Option<ImageNode> {
        let (column, row) = match id {
            "m_bonus" => (1, 1),
            "m_mult" => (2, 1),
            "m_wild" => (3, 1),
            "m_lucky" => (4, 1),
            "m_glass" => (5, 1),
            "m_steel" => (6, 1),
            "m_stone" => (5, 0),
            "m_gold" => (6, 0),
            _ => return None,
        };
        Some(tile(self.card_surfaces.clone()?, column, row))
    }

    pub fn seal(&self, name: &str) -> Option<ImageNode> {
        let (column, row) = match name {
            "Gold" => (2, 0),
            "Purple" => (4, 4),
            "Red" => (5, 4),
            "Blue" => (6, 4),
            _ => return None,
        };
        Some(tile(self.card_surfaces.clone()?, column, row))
    }

    pub fn deck_back(&self, deck: crate::app::navigation::DeckChoice) -> Option<ImageNode> {
        use crate::app::navigation::DeckChoice::*;
        let (x, y) = match deck {
            Red => (0, 0),
            Blue => (0, 2),
            Yellow => (1, 2),
            Green => (2, 2),
            Black => (3, 2),
            Magic => (0, 3),
            Nebula => (3, 0),
            Ghost => (6, 2),
            Abandoned => (3, 3),
            Checkered => (1, 3),
            Zodiac => (3, 4),
            Painted => (4, 3),
            Anaglyph => (2, 4),
            Plasma => (4, 2),
            Erratic => (2, 3),
        };
        Some(tile(self.card_surfaces.clone()?, x, y))
    }

    /// Swirl background cell {2,0} of the 18x18-cell ui_assets sheet (72x36).
    pub fn menu_swirl(&self) -> Option<ImageNode> {
        Some(ImageNode {
            rect: Some(Rect::new(36.0, 0.0, 54.0, 18.0)),
            ..ImageNode::new(self.ui_assets.clone()?)
        })
    }
}

fn square(image: Handle<Image>, size: f32, x: u32, y: u32) -> ImageNode {
    let min = Vec2::new(x as f32, y as f32) * size;
    ImageNode {
        rect: Some(Rect {
            min,
            max: min + Vec2::splat(size),
        }),
        ..ImageNode::new(image)
    }
}

/// Original centre key of an engine enhancement label.
pub(crate) fn enhancement_id(enhancement: &str) -> Option<&'static str> {
    Some(match enhancement {
        "Bonus" => "m_bonus",
        "Mult" => "m_mult",
        "Wild" => "m_wild",
        "Glass" => "m_glass",
        "Steel" => "m_steel",
        "Stone" => "m_stone",
        "Gold" => "m_gold",
        "Lucky" => "m_lucky",
        _ => return None,
    })
}

fn tile(image: Handle<Image>, column: u32, row: u32) -> ImageNode {
    let (x, y) = (column * TILE_WIDTH, row * TILE_HEIGHT);
    ImageNode {
        rect: Some(Rect::new(
            x as f32,
            y as f32,
            (x + TILE_WIDTH) as f32,
            (y + TILE_HEIGHT) as f32,
        )),
        ..ImageNode::new(image)
    }
}

#[cfg(test)]
mod tests;

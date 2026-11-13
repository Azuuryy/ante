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

    pub fn consumable(&self, id: &str) -> Option<ImageNode> {
        let (x, y) = match id {
            "c_fool" => (0, 0),
            "c_magician" => (1, 0),
            "c_high_priestess" => (2, 0),
            "c_empress" => (3, 0),
            "c_emperor" => (4, 0),
            "c_heirophant" => (5, 0),
            "c_lovers" => (6, 0),
            "c_chariot" => (7, 0),
            "c_justice" => (8, 0),
            "c_hermit" => (9, 0),
            "c_wheel_of_fortune" => (0, 1),
            "c_strength" => (1, 1),
            "c_hanged_man" => (2, 1),
            "c_death" => (3, 1),
            "c_temperance" => (4, 1),
            "c_devil" => (5, 1),
            "c_tower" => (6, 1),
            "c_star" => (7, 1),
            "c_moon" => (8, 1),
            "c_sun" => (9, 1),
            "c_judgement" => (0, 2),
            "c_world" => (1, 2),
            "c_soul" => (2, 2),
            "c_eris" => (3, 2),
            "c_ceres" => (8, 2),
            "c_planet_x" => (9, 2),
            "c_mercury" => (0, 3),
            "c_venus" => (1, 3),
            "c_earth" => (2, 3),
            "c_mars" => (3, 3),
            "c_jupiter" => (4, 3),
            "c_saturn" => (5, 3),
            "c_uranus" => (6, 3),
            "c_neptune" => (7, 3),
            "c_pluto" => (8, 3),
            "c_black_hole" => (9, 3),
            "c_familiar" => (0, 4),
            "c_grim" => (1, 4),
            "c_incantation" => (2, 4),
            "c_talisman" => (3, 4),
            "c_aura" => (4, 4),
            "c_wraith" => (5, 4),
            "c_sigil" => (6, 4),
            "c_ouija" => (7, 4),
            "c_ectoplasm" => (8, 4),
            "c_immolate" => (9, 4),
            "c_ankh" => (0, 5),
            "c_deja_vu" => (1, 5),
            "c_hex" => (2, 5),
            "c_trance" => (3, 5),
            "c_medium" => (4, 5),
            "c_cryptid" => (5, 5),
            _ => return None,
        };
        Some(tile(self.consumables.clone()?, x, y))
    }

    pub fn voucher(&self, id: &str) -> Option<ImageNode> {
        let (x, y) = match id {
            "v_overstock_norm" => (0, 0),
            "v_clearance_sale" => (3, 0),
            "v_hone" => (4, 0),
            "v_reroll_surplus" => (0, 2),
            "v_crystal_ball" => (2, 2),
            "v_telescope" => (3, 2),
            "v_grabber" => (5, 0),
            "v_wasteful" => (6, 0),
            "v_tarot_merchant" => (1, 0),
            "v_planet_merchant" => (2, 0),
            "v_seed_money" => (1, 2),
            "v_blank" => (7, 0),
            "v_magic_trick" => (4, 2),
            "v_hieroglyph" => (5, 2),
            "v_directors_cut" => (6, 2),
            "v_paint_brush" => (7, 2),
            "v_overstock_plus" => (0, 1),
            "v_liquidation" => (3, 1),
            "v_glow_up" => (4, 1),
            "v_reroll_glut" => (0, 3),
            "v_omen_globe" => (2, 3),
            "v_observatory" => (3, 3),
            "v_nacho_tong" => (5, 1),
            "v_recyclomancy" => (6, 1),
            "v_tarot_tycoon" => (1, 1),
            "v_planet_tycoon" => (2, 1),
            "v_money_tree" => (1, 3),
            "v_antimatter" => (7, 1),
            "v_illusion" => (4, 3),
            "v_petroglyph" => (5, 3),
            "v_retcon" => (6, 3),
            "v_palette" => (7, 3),
            _ => return None,
        };
        Some(tile(self.vouchers.clone()?, x, y))
    }

    pub fn booster(
        &self,
        family: ante_core::contract::PackFamily,
        size: ante_core::contract::PackSize,
    ) -> Option<ImageNode> {
        use ante_core::contract::{PackFamily::*, PackSize::*};
        let (x, y) = match (family, size) {
            (Arcana, Normal) => (0, 0),
            (Arcana, Jumbo) => (0, 2),
            (Arcana, Mega) => (2, 2),
            (Celestial, Normal) => (0, 1),
            (Celestial, Jumbo) => (0, 3),
            (Celestial, Mega) => (2, 3),
            (Spectral, Normal) => (0, 4),
            (Spectral, Jumbo) => (2, 4),
            (Spectral, Mega) => (3, 4),
            (Standard, Normal) => (0, 6),
            (Standard, Jumbo) => (0, 7),
            (Standard, Mega) => (2, 7),
            (Buffoon, Normal) => (0, 8),
            (Buffoon, Jumbo) => (2, 8),
            (Buffoon, Mega) => (3, 8),
        };
        Some(tile(self.boosters.clone()?, x, y))
    }

    pub fn shop_sign(&self) -> Option<ImageNode> {
        Some(ImageNode {
            rect: Some(Rect::new(0.0, 0.0, 113.0, 57.0)),
            ..ImageNode::new(self.shop_sign.clone()?)
        })
    }

    pub fn joker(&self, id: &str) -> Option<ImageNode> {
        let (x, y) = match id {
            "j_joker" => (0, 0),
            "j_greedy_joker" => (6, 1),
            "j_lusty_joker" => (7, 1),
            "j_wrathful_joker" => (8, 1),
            "j_gluttenous_joker" => (9, 1),
            "j_jolly" => (2, 0),
            "j_zany" => (3, 0),
            "j_mad" => (4, 0),
            "j_crazy" => (5, 0),
            "j_droll" => (6, 0),
            "j_sly" => (0, 14),
            "j_wily" => (1, 14),
            "j_clever" => (2, 14),
            "j_devious" => (3, 14),
            "j_crafty" => (4, 14),
            "j_half" => (7, 0),
            "j_stencil" => (2, 5),
            "j_four_fingers" => (6, 6),
            "j_mime" => (4, 1),
            "j_credit_card" => (5, 1),
            "j_ceremonial" => (5, 5),
            "j_banner" => (1, 2),
            "j_mystic_summit" => (2, 2),
            "j_marble" => (3, 2),
            "j_loyalty_card" => (4, 2),
            "j_8_ball" => (0, 5),
            "j_misprint" => (6, 2),
            "j_dusk" => (4, 7),
            "j_raised_fist" => (8, 2),
            "j_chaos" => (1, 0),
            "j_fibonacci" => (1, 5),
            "j_steel_joker" => (7, 2),
            "j_scary_face" => (2, 3),
            "j_abstract" => (3, 3),
            "j_delayed_grat" => (4, 3),
            "j_hack" => (5, 2),
            "j_pareidolia" => (6, 3),
            "j_gros_michel" => (7, 6),
            "j_even_steven" => (8, 3),
            "j_odd_todd" => (9, 3),
            "j_scholar" => (0, 4),
            "j_business" => (1, 4),
            "j_supernova" => (2, 4),
            "j_ride_the_bus" => (1, 6),
            "j_space" => (3, 5),
            "j_egg" => (0, 10),
            "j_burglar" => (1, 10),
            "j_blackboard" => (2, 10),
            "j_runner" => (3, 10),
            "j_ice_cream" => (4, 10),
            "j_dna" => (5, 10),
            "j_splash" => (6, 10),
            "j_blue_joker" => (7, 10),
            "j_sixth_sense" => (8, 10),
            "j_constellation" => (9, 10),
            "j_hiker" => (0, 11),
            "j_faceless" => (1, 11),
            "j_green_joker" => (2, 11),
            "j_superposition" => (3, 11),
            "j_todo_list" => (4, 11),
            "j_cavendish" => (5, 11),
            "j_card_sharp" => (6, 11),
            "j_red_card" => (7, 11),
            "j_madness" => (8, 11),
            "j_square" => (9, 11),
            "j_seance" => (0, 12),
            "j_riff_raff" => (1, 12),
            "j_vampire" => (2, 12),
            "j_shortcut" => (3, 12),
            "j_hologram" => (4, 12),
            "j_vagabond" => (5, 12),
            "j_baron" => (6, 12),
            "j_cloud_9" => (7, 12),
            "j_rocket" => (8, 12),
            "j_obelisk" => (9, 12),
            "j_midas_mask" => (0, 13),
            "j_luchador" => (1, 13),
            "j_photograph" => (2, 13),
            "j_gift" => (3, 13),
            "j_turtle_bean" => (4, 13),
            "j_erosion" => (5, 13),
            "j_reserved_parking" => (6, 13),
            "j_mail" => (7, 13),
            "j_to_the_moon" => (8, 13),
            "j_hallucination" => (9, 13),
            "j_fortune_teller" => (7, 5),
            "j_juggler" => (0, 1),
            "j_drunkard" => (1, 1),
            "j_stone" => (9, 0),
            "j_golden" => (9, 2),
            "j_lucky_cat" => (5, 14),
            "j_baseball" => (6, 14),
            "j_bull" => (7, 14),
            "j_diet_cola" => (8, 14),
            "j_trading" => (9, 14),
            "j_flash" => (0, 15),
            "j_popcorn" => (1, 15),
            "j_trousers" => (4, 15),
            "j_ancient" => (7, 15),
            "j_ramen" => (2, 15),
            "j_walkie_talkie" => (8, 15),
            "j_selzer" => (3, 15),
            "j_castle" => (9, 15),
            "j_smiley" => (6, 15),
            "j_campfire" => (5, 15),
            "j_ticket" => (5, 3),
            "j_mr_bones" => (3, 4),
            "j_acrobat" => (2, 1),
            "j_sock_and_buskin" => (3, 1),
            "j_swashbuckler" => (9, 5),
            "j_troubadour" => (0, 2),
            "j_certificate" => (8, 8),
            "j_smeared" => (4, 6),
            "j_throwback" => (5, 7),
            "j_hanging_chad" => (9, 6),
            "j_rough_gem" => (9, 7),
            "j_bloodstone" => (0, 8),
            "j_arrowhead" => (1, 8),
            "j_onyx_agate" => (2, 8),
            "j_glass" => (1, 3),
            "j_ring_master" => (6, 5),
            "j_flower_pot" => (0, 6),
            "j_blueprint" => (0, 3),
            "j_wee" => (0, 0),
            "j_merry_andy" => (8, 0),
            "j_oops" => (5, 6),
            "j_idol" => (6, 7),
            "j_seeing_double" => (4, 4),
            "j_matador" => (4, 5),
            "j_hit_the_road" => (8, 5),
            "j_duo" => (5, 4),
            "j_trio" => (6, 4),
            "j_family" => (7, 4),
            "j_order" => (8, 4),
            "j_tribe" => (9, 4),
            "j_stuntman" => (8, 6),
            "j_invisible" => (1, 7),
            "j_brainstorm" => (7, 7),
            "j_satellite" => (8, 7),
            "j_shoot_the_moon" => (2, 6),
            "j_drivers_license" => (0, 7),
            "j_cartomancer" => (7, 3),
            "j_astronomer" => (2, 7),
            "j_burnt" => (3, 7),
            "j_bootstraps" => (9, 8),
            "j_caino" => (3, 8),
            "j_triboulet" => (4, 8),
            "j_yorick" => (5, 8),
            "j_chicot" => (6, 8),
            "j_perkeo" => (7, 8),
            _ => return None,
        };
        Some(tile(self.jokers.clone()?, x, y))
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

pub fn load(
    source: Res<AtlasSource>,
    mut atlases: ResMut<Atlases>,
    mut images: ResMut<Assets<Image>>,
    mut fonts: ResMut<Assets<Font>>,
    mut metrics: ResMut<FrameMetrics>,
) {
    let Some(root) = &source.0 else { return };
    let start = crate::services::diagnostics::measurement();
    let mut load_one = |name| {
        let path = format!("resources/textures/1x/{name}");
        let dimensions = LOCAL_FILES
            .iter()
            .find(|(file, _)| *file == path)
            .and_then(|(_, dimensions)| *dimensions)
            .expect("known atlas");
        match read_atlas(root, name, dimensions) {
            Ok(image) => {
                crate::journal_event!("atlas_loaded", json!({"file": name}));
                Some(images.add(image))
            }
            Err(_error) => {
                crate::journal_event!(
                    "atlas_fallback",
                    json!({"file": name, "error": _error.to_string()})
                );
                None
            }
        }
    };
    atlases.cards = load_one("8BitDeck.png");
    atlases.card_surfaces = load_one("Enhancers.png");
    atlases.boosters = load_one("boosters.png");
    atlases.vouchers = load_one("Vouchers.png");
    atlases.jokers = load_one("Jokers.png");
    atlases.consumables = load_one("Tarots.png");
    atlases.shop_sign = load_one("ShopSignAnimation.png");
    atlases.blind_chips = load_one("BlindChips.png");
    atlases.tags = load_one("tags.png");
    atlases.stake_chips = load_one("chips.png");
    atlases.logo = load_one("balatro.png");
    atlases.ui_assets = load_one("ui_assets.png");
    atlases.font = match read_local_font(root) {
        Ok(font) => Some(fonts.add(font)),
        Err(_error) => {
            crate::journal_event!(
                "asset_fallback",
                json!({"file": "m6x11plus.ttf", "error": _error.to_string()}),
            );
            None
        }
    };
    atlases.status = Some(match (atlases.cards.is_some(), atlases.jokers.is_some()) {
        (true, true) => "Local card faces and joker atlas loaded",
        (true, false) => "Local card faces loaded - joker placeholders",
        (false, true) => "Card placeholders - local joker atlas loaded",
        (false, false) => "Local atlases unavailable - using placeholders",
    });
    metrics.record_assets(start.elapsed());
}

pub(crate) const LOCAL_FILES: &[(&str, Option<(u32, u32)>)] = &[
    ("resources/textures/1x/8BitDeck.png", Some((923, 380))),
    ("resources/textures/1x/Enhancers.png", Some((497, 475))),
    ("resources/textures/1x/boosters.png", Some((284, 855))),
    ("resources/textures/1x/Vouchers.png", Some((639, 380))),
    ("resources/textures/1x/Jokers.png", Some((710, 1520))),
    ("resources/textures/1x/Tarots.png", Some((710, 570))),
    (
        "resources/textures/1x/ShopSignAnimation.png",
        Some((452, 57)),
    ),
    ("resources/textures/1x/BlindChips.png", Some((714, 1054))),
    ("resources/textures/1x/tags.png", Some((204, 170))),
    ("resources/textures/1x/chips.png", Some((145, 58))),
    ("resources/textures/1x/balatro.png", Some((333, 216))),
    ("resources/textures/1x/ui_assets.png", Some((72, 36))),
    ("resources/fonts/m6x11plus.ttf", None),
];

pub(crate) fn read_local_file(root: &Path, relative: &str) -> io::Result<Vec<u8>> {
    let root = root.canonicalize()?;
    let path = root.join(relative).canonicalize()?;
    if !path.starts_with(&root) || !path.is_file() {
        return Err(io::Error::other(
            "Asset must be a regular file inside the selected folder",
        ));
    }
    let mut bytes = Vec::new();
    File::open(path)?
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(io::Error::other("Asset exceeds the file size limit"));
    }
    Ok(bytes)
}

pub(crate) fn validate_local_file(bytes: &[u8], dimensions: Option<(u32, u32)>) -> io::Result<()> {
    match dimensions {
        Some(dimensions) => decode_atlas(bytes, dimensions).map(|_| ()),
        None => validate_font(bytes),
    }
}

pub(crate) fn read_local_font(root: &Path) -> io::Result<Font> {
    let bytes = read_local_file(root, "resources/fonts/m6x11plus.ttf")?;
    validate_font(&bytes)?;
    Ok(Font::from_bytes(bytes))
}

fn validate_font(bytes: &[u8]) -> io::Result<()> {
    if bytes.len() as u64 > MAX_BYTES || skrifa::FontRef::new(bytes).is_err() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid local font",
        ));
    }
    Ok(())
}

fn read_atlas(root: &Path, name: &str, dimensions: (u32, u32)) -> io::Result<Image> {
    let bytes = read_local_file(root, &format!("resources/textures/1x/{name}"))?;
    decode_atlas(&bytes, dimensions)
}

fn decode_atlas(bytes: &[u8], (width, height): (u32, u32)) -> io::Result<Image> {
    validate_png(bytes, width, height)?;
    Image::from_buffer(
        bytes,
        ImageType::Extension("png"),
        CompressedImageFormats::NONE,
        true,
        ImageSampler::nearest(),
        RenderAssetUsages::default(),
    )
    .map_err(io::Error::other)
}

fn validate_png(bytes: &[u8], width: u32, height: u32) -> io::Result<()> {
    if bytes.len() as u64 > MAX_BYTES
        || bytes.get(..16) != Some(b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR")
        || bytes.get(16..20) != Some(&width.to_be_bytes())
        || bytes.get(20..24) != Some(&height.to_be_bytes())
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Unexpected PNG dimensions or format",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deck_previews_use_distinct_in_bounds_atlas_cells() {
        use crate::app::navigation::DeckChoice;
        let atlases = Atlases {
            card_surfaces: Some(Handle::default()),
            ..default()
        };
        let mut rects = Vec::new();
        for choice in DeckChoice::ALL {
            let rect = atlases.deck_back(choice).unwrap().rect.unwrap();
            assert!(rect.min.x >= 0.0 && rect.min.y >= 0.0);
            assert!(rect.max.x <= 497.0 && rect.max.y <= 475.0);
            assert!(!rects.contains(&rect));
            rects.push(rect);
        }
    }

    #[test]
    #[ignore = "requires an explicitly provided local installation"]
    fn decode_local_card_and_joker_atlases() {
        let root = PathBuf::from(std::env::var_os("ANTE_TEST_ASSET_DIR").unwrap());
        assert!(read_atlas(&root, "Vouchers.png", (639, 380)).is_ok());
        let deck = read_atlas(&root, "8BitDeck.png", (923, 380)).unwrap();
        let jokers = read_atlas(&root, "Jokers.png", (710, 1520)).unwrap();
        assert_eq!(deck.width(), 923);
        assert_eq!(jokers.height(), 1520);
        let consumables = read_atlas(&root, "Tarots.png", (710, 570)).unwrap();
        for (file, size) in [
            ("BlindChips.png", (714, 1054)),
            ("tags.png", (204, 170)),
            ("chips.png", (145, 58)),
        ] {
            assert!(read_atlas(&root, file, size).is_ok(), "{file}");
        }
        assert_eq!((consumables.width(), consumables.height()), (710, 570));
        let surfaces = read_atlas(&root, "Enhancers.png", (497, 475)).unwrap();
        assert_eq!((surfaces.width(), surfaces.height()), (497, 475));
        let logo = read_atlas(&root, "balatro.png", (333, 216)).unwrap();
        assert_eq!((logo.width(), logo.height()), (333, 216));
        let ui_assets = read_atlas(&root, "ui_assets.png", (72, 36)).unwrap();
        assert_eq!((ui_assets.width(), ui_assets.height()), (72, 36));
        assert!(read_local_font(&root).is_ok());
    }

    #[test]
    fn menu_swirl_requires_the_sheet_and_uses_cell_2_0() {
        assert!(Atlases::default().menu_swirl().is_none());
        let atlases = Atlases {
            ui_assets: Some(Handle::default()),
            ..default()
        };
        let swirl = atlases.menu_swirl().unwrap();
        assert_eq!(swirl.rect, Some(Rect::new(36.0, 0.0, 54.0, 18.0)));
    }

    #[test]
    fn rejects_missing_and_invalid_assets() {
        let missing = std::env::temp_dir().join("ante-nonexistent-asset-source");
        assert!(read_atlas(&missing, "balatro.png", (333, 216)).is_err());
        assert!(read_local_font(&missing).is_err());
        assert!(validate_font(b"invalid font").is_err());
    }

    #[test]
    fn rejects_invalid_atlas_before_decode() {
        assert!(validate_png(b"not a PNG", 923, 380).is_err());
        let mut header = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        header.extend_from_slice(&u32::MAX.to_be_bytes());
        header.extend_from_slice(&380_u32.to_be_bytes());
        assert!(validate_png(&header, 923, 380).is_err());
    }
}

use crate::instance::Instance;
use crate::node_id::NodeId;
use crate::pools;
use crate::resolve;
use balatro_types::{
    BossBlind, Card, Consumable, Edition, Jokers, PackCategory, PackSize, Seal, Spectral, Tag,
    Voucher,
};

mod consumables;
mod packs;

pub enum ShopItem {
    Joker(Jokers),
    Consumable(Consumable),
    PlayingCard,
}

/// Given a drawn base voucher, returns its upgrade tier if one exists.
/// `VOUCHERS` is laid out as (base, upgrade) pairs at consecutive indices.
pub fn voucher_upgrade(voucher: Voucher) -> Option<Voucher> {
    let name = voucher.name();
    let idx = pools::VOUCHERS.iter().position(|&n| n == name)?;
    if idx % 2 == 0 {
        resolve::resolve_voucher(pools::VOUCHERS[idx + 1])
    } else {
        None
    }
}

/// Decodes a drawn pack name into its typed category and size.
fn parse_pack_name(name: &str) -> (PackCategory, PackSize) {
    let (category_str, size) = if let Some(rest) = name.strip_prefix("Mega ") {
        (rest, PackSize::Mega)
    } else if let Some(rest) = name.strip_prefix("Jumbo ") {
        (rest, PackSize::Jumbo)
    } else {
        (name, PackSize::Normal)
    };
    let category = match category_str {
        "Arcana Pack" => PackCategory::Arcana,
        "Celestial Pack" => PackCategory::Celestial,
        "Buffoon Pack" => PackCategory::Buffoon,
        "Standard Pack" => PackCategory::Standard,
        "Spectral Pack" => PackCategory::Spectral,
        other => panic!("pack name {other:?} has no balatro_types::PackCategory match"),
    };
    (category, size)
}

/// Card count for a pack: Buffoon/Spectral hold 2 (Normal) / 4 (else);
/// every other category holds 3 (Normal) / 5 (else).
pub fn pack_card_count(category: PackCategory, size: PackSize) -> i32 {
    match (category, size) {
        (PackCategory::Buffoon | PackCategory::Spectral, PackSize::Normal) => 2,
        (PackCategory::Buffoon | PackCategory::Spectral, _) => 4,
        (_, PackSize::Normal) => 3,
        _ => 5,
    }
}

impl Instance {
    /// `fresh_profile` gates profile-level achievement locks; `fresh_run`
    /// gates locks whose requirement is inherently in-run.
    pub fn init_locks(&mut self, ante: i32, fresh_profile: bool, fresh_run: bool) {
        if ante < 2 {
            for name in [
                "The Mouth",
                "The Fish",
                "The Wall",
                "The House",
                "The Mark",
                "The Wheel",
                "The Arm",
                "The Water",
                "The Needle",
                "The Flint",
                "Negative Tag",
                "Standard Tag",
                "Meteor Tag",
                "Buffoon Tag",
                "Handy Tag",
                "Garbage Tag",
                "Ethereal Tag",
                "Top-up Tag",
                "Orbital Tag",
            ] {
                self.lock(name);
            }
        }
        if ante < 3 {
            self.lock("The Tooth");
            self.lock("The Eye");
        }
        if ante < 4 {
            self.lock("The Plant");
        }
        if ante < 5 {
            self.lock("The Serpent");
        }
        if ante < 6 {
            self.lock("The Ox");
        }
        if fresh_profile {
            for name in [
                "Negative Tag",
                "Foil Tag",
                "Holographic Tag",
                "Polychrome Tag",
                "Rare Tag",
                "Golden Ticket",
                "Mr. Bones",
                "Acrobat",
                "Sock and Buskin",
                "Swashbuckler",
                "Troubadour",
                "Certificate",
                "Smeared Joker",
                "Throwback",
                "Hanging Chad",
                "Rough Gem",
                "Bloodstone",
                "Arrowhead",
                "Onyx Agate",
                "Glass Joker",
                "Showman",
                "Flower Pot",
                "Blueprint",
                "Wee Joker",
                "Merry Andy",
                "Oops! All 6s",
                "The Idol",
                "Seeing Double",
                "Matador",
                "Hit the Road",
                "The Duo",
                "The Trio",
                "The Family",
                "The Order",
                "The Tribe",
                "Stuntman",
                "Invisible Joker",
                "Brainstorm",
                "Satellite",
                "Shoot the Moon",
                "Driver's License",
                "Cartomancer",
                "Astronomer",
                "Burnt Joker",
                "Bootstraps",
                "Overstock Plus",
                "Liquidation",
                "Glow Up",
                "Reroll Glut",
                "Omen Globe",
                "Observatory",
                "Nacho Tong",
                "Recyclomancy",
                "Tarot Tycoon",
                "Planet Tycoon",
                "Money Tree",
                "Antimatter",
                "Illusion",
                "Petroglyph",
                "Retcon",
                "Palette",
            ] {
                self.lock(name);
            }
        }
        if fresh_run {
            for name in [
                "Planet X",
                "Ceres",
                "Eris",
                "Five of a Kind",
                "Flush House",
                "Flush Five",
                "Stone Joker",
                "Steel Joker",
                "Glass Joker",
                "Golden Ticket",
                "Lucky Cat",
                "Cavendish",
                "Overstock Plus",
                "Liquidation",
                "Glow Up",
                "Reroll Glut",
                "Omen Globe",
                "Observatory",
                "Nacho Tong",
                "Recyclomancy",
                "Tarot Tycoon",
                "Planet Tycoon",
                "Money Tree",
                "Antimatter",
                "Illusion",
                "Petroglyph",
                "Retcon",
                "Palette",
            ] {
                self.lock(name);
            }
        }
    }

    /// Ante-gated unlocks (`functions.hpp::initUnlocks`).
    pub fn init_unlocks(&mut self, ante: i32, fresh_profile: bool) {
        if ante == 2 {
            for name in [
                "The Mouth",
                "The Fish",
                "The Wall",
                "The House",
                "The Mark",
                "The Wheel",
                "The Arm",
                "The Water",
                "The Needle",
                "The Flint",
                "Standard Tag",
                "Meteor Tag",
                "Buffoon Tag",
                "Handy Tag",
                "Garbage Tag",
                "Ethereal Tag",
                "Top-up Tag",
                "Orbital Tag",
            ] {
                self.unlock(name);
            }
            if !fresh_profile {
                self.unlock("Negative Tag");
            }
        }
        if ante == 3 {
            self.unlock("The Tooth");
            self.unlock("The Eye");
        }
        if ante == 4 {
            self.unlock("The Plant");
        }
        if ante == 5 {
            self.unlock("The Serpent");
        }
        if ante == 6 {
            self.unlock("The Ox");
        }
    }
}

#[cfg(test)]
mod tests;

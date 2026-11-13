use std::sync::LazyLock;

use balatro_rs::tag::Tag;
use strum::IntoEnumIterator;

use super::*;

pub(super) fn voucher_name(voucher: Voucher) -> &'static str {
    VOUCHERS
        .iter()
        .find(|item| item.id.0 == voucher.id())
        .unwrap()
        .name
        .as_str()
}

pub(super) static VOUCHERS: LazyLock<Vec<Content>> = LazyLock::new(|| {
    Voucher::iter()
        .map(|voucher| Content {
            id: ContentId(voucher.id()),
            name: voucher.name().to_owned(),
            description: voucher.description().to_owned(),
            cost: voucher.cost(),
            supported: true,
            unlock: unlock_condition(voucher.id()),
        })
        .collect()
});

pub(super) fn voucher_content(voucher: Voucher) -> &'static Content {
    VOUCHERS
        .iter()
        .find(|item| item.id.0 == voucher.id())
        .unwrap()
}

pub(super) fn consumable_content(card: balatro_rs::consumable::Consumable) -> Content {
    Content {
        id: ContentId(card.id()),
        name: card.name().to_owned(),
        description: consumable_description(card),
        cost: card.cost(),
        supported: true,
        unlock: None,
    }
}

pub(super) static TAROTS: LazyLock<Vec<Content>> = LazyLock::new(|| {
    balatro_rs::tarot::Tarot::iter()
        .map(|card| consumable_content(balatro_rs::consumable::Consumable::Tarot(card)))
        .collect()
});

pub(super) static PLANETS: LazyLock<Vec<Content>> = LazyLock::new(|| {
    balatro_rs::planet::Planets::iter()
        .map(|card| consumable_content(balatro_rs::consumable::Consumable::Planet(card)))
        .collect()
});

pub(super) static SPECTRALS: LazyLock<Vec<Content>> = LazyLock::new(|| {
    balatro_rs::spectral::Spectral::iter()
        .map(|card| consumable_content(balatro_rs::consumable::Consumable::Spectral(card)))
        .collect()
});

/// Pack contents and price by size: choose 1 of 3, 1 of 5 or 2 of 5 (2, 4 and 4 for
/// Spectral and Buffoon packs), for $4, $6 or $8.
pub(super) static BOOSTER_CONTENT: LazyLock<Vec<Content>> = LazyLock::new(|| {
    use crate::contract::{BOOSTERS, PackFamily, PackSize};
    BOOSTERS
        .iter()
        .map(|&(id, name, family, size)| {
            let small = matches!(family, PackFamily::Spectral | PackFamily::Buffoon);
            let (choose, shown, cost) = match size {
                PackSize::Normal => (1, if small { 2 } else { 3 }, 4),
                PackSize::Jumbo => (1, if small { 4 } else { 5 }, 6),
                PackSize::Mega => (2, if small { 4 } else { 5 }, 8),
            };
            let what = match family {
                PackFamily::Arcana => "Tarot cards, used at once",
                PackFamily::Celestial => "Planet cards, used at once",
                PackFamily::Spectral => "Spectral cards, used at once",
                PackFamily::Standard => "playing cards added to your deck",
                PackFamily::Buffoon => "Jokers",
            };
            Content {
                id: ContentId(id),
                name: name.to_owned(),
                description: format!("Pick {choose} of {shown} {what}."),
                cost,
                supported: true,
                unlock: None,
            }
        })
        .collect()
});

pub(super) static BLINDS: LazyLock<Vec<Content>> = LazyLock::new(|| {
    use balatro_types::BossBlind;
    let mut blinds = vec![
        Content {
            id: ContentId("bl_small"),
            name: "Small Blind".to_owned(),
            description: "No special effect.".to_owned(),
            cost: 0,
            supported: true,
            unlock: None,
        },
        Content {
            id: ContentId("bl_big"),
            name: "Big Blind".to_owned(),
            description: "No special effect.".to_owned(),
            cost: 0,
            supported: true,
            unlock: None,
        },
    ];
    blinds.extend(BossBlind::iter().map(|boss| Content {
        id: ContentId(boss.id()),
        name: boss.name().to_owned(),
        description: boss.description().to_owned(),
        cost: 0,
        supported: true,
        unlock: None,
    }));
    blinds
});

pub(super) fn rule(id: &'static str, name: &str, description: &str) -> Content {
    Content {
        id: ContentId(id),
        name: name.to_owned(),
        description: description.to_owned(),
        cost: 0,
        supported: true,
        unlock: None,
    }
}

/// Ante's own summaries of each rule, matching the tooltip side boxes.
pub(super) static ENHANCEMENTS: LazyLock<Vec<Content>> = LazyLock::new(|| {
    vec![
        rule("m_bonus", "Bonus Card", "+30 Chips"),
        rule("m_mult", "Mult Card", "+4 Mult"),
        rule("m_wild", "Wild Card", "Counts as every suit"),
        rule(
            "m_glass",
            "Glass Card",
            "X2 Mult, 1 in 4 chance to break after scoring",
        ),
        rule("m_steel", "Steel Card", "X1.5 Mult while held in hand"),
        rule(
            "m_stone",
            "Stone Card",
            "+50 Chips; has no rank or suit and always scores",
        ),
        rule(
            "m_gold",
            "Gold Card",
            "Earn $3 if held in hand at end of round",
        ),
        rule(
            "m_lucky",
            "Lucky Card",
            "+20 Mult on a 1 in 5 chance; $20 on a 1 in 15 chance",
        ),
    ]
});

pub(super) static EDITIONS: LazyLock<Vec<Content>> = LazyLock::new(|| {
    vec![
        rule("e_foil", "Foil", "+50 Chips"),
        rule("e_holo", "Holographic", "+10 Mult"),
        rule("e_polychrome", "Polychrome", "X1.5 Mult"),
        rule("e_negative", "Negative", "+1 Joker slot"),
    ]
});

pub(super) static SEALS: LazyLock<Vec<Content>> = LazyLock::new(|| {
    vec![
        rule("gold_seal", "Gold Seal", "Played and scoring: earn $3"),
        rule("red_seal", "Red Seal", "This card triggers 1 more time"),
        rule(
            "blue_seal",
            "Blue Seal",
            "Held in hand at end of round: creates the Planet of the last played hand",
        ),
        rule(
            "purple_seal",
            "Purple Seal",
            "Discarding this card creates a Tarot card",
        ),
    ]
});

pub(super) static TAGS: LazyLock<Vec<Content>> = LazyLock::new(|| {
    Tag::iter()
        .map(|tag| Content {
            id: ContentId(tag.id()),
            name: tag.name().to_owned(),
            description: tag.description().to_owned(),
            cost: 0,
            supported: true,
            unlock: None,
        })
        .collect()
});
/// Collection unlock text for items that are not available from the start.
/// Conditions are paraphrased from the reference rules; they are display
/// metadata only and do not drive unlock progression yet.
pub fn unlock_condition(id: &str) -> Option<&'static str> {
    Some(match id {
        "j_ticket" => "Play a hand containing 5 Gold cards.",
        "j_mr_bones" => "Lose 5 runs.",
        "j_acrobat" => "Play 200 hands.",
        "j_sock_and_buskin" => "Play 300 face cards.",
        "j_swashbuckler" => "Sell 20 Jokers.",
        "j_troubadour" => "Win 5 consecutive rounds in a single hand.",
        "j_certificate" => "Score with Gold cards in a hand.",
        "j_smeared" => "Have 3 Wild cards in your deck.",
        "j_throwback" => "Continue a saved run.",
        "j_hanging_chad" => "Defeat a Boss Blind with a High Card hand.",
        "j_rough_gem" => "Have 30 Diamonds in your deck.",
        "j_bloodstone" => "Have 30 Hearts in your deck.",
        "j_arrowhead" => "Have 30 Spades in your deck.",
        "j_onyx_agate" => "Have 30 Clubs in your deck.",
        "j_glass" => "Have 5 Glass cards in your deck.",
        "j_ring_master" => "Reach Ante 4.",
        "j_flower_pot" => "Reach Ante 8.",
        "j_blueprint" => "Win a run.",
        "j_wee" => "Win a run in 18 rounds or fewer.",
        "j_merry_andy" => "Win a run in 12 rounds or fewer.",
        "j_oops" => "Score 10,000 chips in one hand.",
        "j_idol" => "Score 1,000,000 chips in one hand.",
        "j_seeing_double" => "Play four 7 of Clubs in one hand.",
        "j_matador" => "Defeat a Boss Blind in one hand without discarding.",
        "j_hit_the_road" => "Discard 5 Jacks at once.",
        "j_duo" => "Win a run without playing a Pair.",
        "j_trio" => "Win a run without playing Three of a Kind.",
        "j_family" => "Win a run without playing Four of a Kind.",
        "j_order" => "Win a run without playing a Straight.",
        "j_tribe" => "Win a run without playing a Flush.",
        "j_stuntman" => "Score 100,000,000 chips in one hand.",
        "j_invisible" => "Win a run with 4 or fewer Joker slots.",
        "j_brainstorm" => "Discard a Royal Flush.",
        "j_satellite" => "Have $400 at once.",
        "j_shoot_the_moon" => "Play every Heart from your deck in one hand.",
        "j_drivers_license" => "Have 16 enhanced cards in your deck.",
        "j_cartomancer" => "Discover 22 Tarot cards.",
        "j_astronomer" => "Discover 12 Planet cards.",
        "j_burnt" => "Sell 50 cards.",
        "j_bootstraps" => "Have 2 Polychrome Jokers.",
        "j_caino" | "j_triboulet" | "j_yorick" | "j_chicot" | "j_perkeo" => {
            "Discover it from a Spectral pack."
        }
        "v_overstock_plus" => "Spend $2,500 in the shop.",
        "v_liquidation" => "Redeem 10 Vouchers in one run.",
        "v_glow_up" => "Have 5 Jokers with editions.",
        "v_reroll_glut" => "Reroll the shop 100 times.",
        "v_omen_globe" => "Use 25 Tarot cards.",
        "v_observatory" => "Use 25 Planet cards.",
        "v_nacho_tong" => "Play 2,500 cards.",
        "v_recyclomancy" => "Discard 2,500 cards.",
        "v_tarot_tycoon" => "Buy 50 Tarot cards.",
        "v_planet_tycoon" => "Buy 50 Planet cards.",
        "v_money_tree" => "Reach maximum interest in 10 consecutive rounds.",
        "v_antimatter" => "Redeem the Blank Voucher 10 times.",
        "v_illusion" => "Buy 20 playing cards.",
        "v_petroglyph" => "Reach Ante 12.",
        "v_retcon" => "Discover 25 Blinds.",
        "v_palette" => "Win a round with a hand size of 5 or less.",
        _ => return None,
    })
}

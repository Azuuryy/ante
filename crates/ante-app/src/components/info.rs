//! Side boxes of a tooltip, the original `info_queue`: editions, seals, stickers and the
//! enhancements or editions a card refers to. The wording is Ante's own summary of each rule.

/// One side box: a title and a short rule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Info {
    pub(crate) title: &'static str,
    pub(crate) body: &'static str,
}

const fn info(title: &'static str, body: &'static str) -> Info {
    Info { title, body }
}

/// Edition box for an engine edition label; a negative consumable adds a consumable slot.
pub(crate) fn edition(name: &str, consumable: bool) -> Option<Info> {
    Some(match name {
        "Foil" => info("Foil", "+50 Chips"),
        "Holographic" | "Holo" => info("Holographic", "+10 Mult"),
        "Polychrome" => info("Polychrome", "X1.5 Mult"),
        "Negative" if consumable => info("Negative", "+1 consumable slot"),
        "Negative" => info("Negative", "+1 Joker slot"),
        _ => return None,
    })
}

/// Seal box for an engine seal label.
pub(crate) fn seal(name: &str) -> Option<Info> {
    Some(match name {
        "Gold" => info("Gold Seal", "Played and scoring: earn $3"),
        "Red" => info("Red Seal", "This card triggers 1 more time"),
        "Blue" => info(
            "Blue Seal",
            "Held in hand at end of round: creates the Planet of the last played hand",
        ),
        "Purple" => info("Purple Seal", "Discarding this card creates a Tarot card"),
        _ => return None,
    })
}

/// Enhancement box for an engine enhancement label.
pub(crate) fn enhancement(name: &str) -> Option<Info> {
    Some(match name {
        "Bonus" => info("Bonus Card", "+30 Chips"),
        "Mult" => info("Mult Card", "+4 Mult"),
        "Wild" => info("Wild Card", "Counts as every suit"),
        "Glass" => info(
            "Glass Card",
            "X2 Mult, 1 in 4 chance to break after scoring",
        ),
        "Steel" => info("Steel Card", "X1.5 Mult while held in hand"),
        "Stone" => info(
            "Stone Card",
            "+50 Chips; has no rank or suit and always scores",
        ),
        "Gold" => info("Gold Card", "Earn $3 if held in hand at end of round"),
        "Lucky" => info(
            "Lucky Card",
            "+20 Mult on a 1 in 5 chance; $20 on a 1 in 15 chance",
        ),
        _ => return None,
    })
}

/// Sticker boxes for the adapter's " · " separated sticker list.
pub(crate) fn stickers(list: &str) -> Vec<Info> {
    list.split(" · ")
        .filter_map(|name| {
            Some(match name.trim() {
                "Eternal" => info("Eternal", "Never sold and never destroyed"),
                "Perishable" => info("Perishable", "Becomes debuffed 5 rounds after purchase"),
                "Rental" => info("Rental", "Costs $3 at the end of every round"),
                _ => return None,
            })
        })
        .collect()
}

/// Boxes a card shows for what its own effect mentions, like the original
/// `generate_card_ui` (Stone Joker shows the Stone Card, The Chariot the Steel Card).
pub(crate) fn references(kind: &str) -> Vec<Info> {
    let enhanced = |name| enhancement(name).into_iter().collect();
    let editions = || {
        ["Foil", "Holographic", "Polychrome"]
            .into_iter()
            .filter_map(|name| edition(name, false))
            .collect()
    };
    match kind {
        "j_marble" | "j_stone" | "c_tower" => enhanced("Stone"),
        "j_steel_joker" | "c_chariot" => enhanced("Steel"),
        "j_glass" | "c_justice" => enhanced("Glass"),
        "j_ticket" | "j_midas_mask" | "c_devil" => enhanced("Gold"),
        "j_lucky_cat" | "c_magician" => enhanced("Lucky"),
        "c_empress" => enhanced("Mult"),
        "c_heirophant" => enhanced("Bonus"),
        "c_lovers" => enhanced("Wild"),
        "c_wheel_of_fortune" | "c_aura" => editions(),
        "c_hex" | "tag_polychrome" => edition("Polychrome", false).into_iter().collect(),
        "c_ectoplasm" | "tag_negative" => edition("Negative", false).into_iter().collect(),
        "tag_foil" => edition("Foil", false).into_iter().collect(),
        "tag_holo" => edition("Holographic", false).into_iter().collect(),
        "j_diet_cola" => vec![info("Double Tag", "Gives a copy of the next Tag taken")],
        "j_perkeo" => edition("Negative", true).into_iter().collect(),
        "tag_charm" => vec![info(
            "Mega Arcana Pack",
            "Pick 2 of 5 Tarot cards, used at once",
        )],
        "tag_meteor" => vec![info(
            "Mega Celestial Pack",
            "Pick 2 of 5 Planet cards, used at once",
        )],
        "tag_ethereal" => vec![info(
            "Spectral Pack",
            "Pick 1 of 2 Spectral cards, used at once",
        )],
        "tag_standard" => vec![info(
            "Mega Standard Pack",
            "Pick 2 of 5 playing cards added to your deck",
        )],
        "tag_buffoon" => vec![info("Mega Buffoon Pack", "Pick 2 of 4 Jokers")],
        "c_talisman" => seal("Gold").into_iter().collect(),
        "c_deja_vu" => seal("Red").into_iter().collect(),
        "c_trance" => seal("Blue").into_iter().collect(),
        "c_medium" => seal("Purple").into_iter().collect(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_attributes_have_no_box() {
        assert_eq!(edition("Base", false), None);
        assert_eq!(seal("None"), None);
        assert_eq!(enhancement("None"), None);
        assert!(stickers("").is_empty());
    }

    #[test]
    fn negative_depends_on_the_card_type() {
        assert_eq!(edition("Negative", false).unwrap().body, "+1 Joker slot");
        assert_eq!(
            edition("Negative", true).unwrap().body,
            "+1 consumable slot"
        );
    }

    #[test]
    fn stickers_split_like_the_adapter_lists_them() {
        let boxes = stickers("Eternal · Rental");
        assert_eq!(
            boxes.iter().map(|info| info.title).collect::<Vec<_>>(),
            ["Eternal", "Rental"]
        );
    }

    #[test]
    fn cards_refer_to_what_their_effect_mentions() {
        assert_eq!(references("j_marble")[0].title, "Stone Card");
        assert_eq!(references("c_chariot")[0].title, "Steel Card");
        assert_eq!(references("c_aura").len(), 3);
        assert_eq!(references("c_talisman")[0].title, "Gold Seal");
        assert!(references("j_joker").is_empty());
        assert_eq!(references("j_perkeo")[0].body, "+1 consumable slot");
        assert_eq!(references("tag_charm")[0].title, "Mega Arcana Pack");
    }
}

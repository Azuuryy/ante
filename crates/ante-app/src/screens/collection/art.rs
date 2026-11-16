use super::*;

/// Sprite of a collection entry: its own atlas cell, or the card it decorates.
pub(super) fn collection_sprite(
    atlases: &Atlases,
    group: ContentGroup,
    id: &str,
) -> Option<ImageNode> {
    match group {
        ContentGroup::Jokers => atlases.joker(id),
        ContentGroup::Vouchers => atlases.voucher(id),
        ContentGroup::Tags => atlases.tag_icon(id),
        ContentGroup::Tarots | ContentGroup::Planets | ContentGroup::Spectrals => {
            atlases.consumable(id)
        }
        ContentGroup::Boosters => ante_core::contract::BOOSTERS
            .iter()
            .find(|(booster, ..)| *booster == id)
            .and_then(|&(_, _, family, size)| atlases.booster(family, size)),
        ContentGroup::Blinds => atlases.blind_chip(id),
        ContentGroup::Enhancements => atlases.enhancement(id),
        // `e_*` centres use the Joker atlas at {0, 0}: the plain Joker.
        ContentGroup::Editions => atlases.joker("j_joker"),
        ContentGroup::Seals => atlases.card_base(),
    }
}

/// Fills a card of `size` px: its art like the original collection draws it (editions on
/// the plain Joker with their shader, seals stamped on a plain card), or the undiscovered
/// or locked centre with the "?" over it.
pub(super) fn collection_card_art(
    art: &mut ChildSpawnerCommands,
    atlases: &Atlases,
    group: ContentGroup,
    id: &str,
    (discovered, locked): (bool, bool),
    size: Vec2,
) {
    let hidden = locked || (!discovered && hides_undiscovered(group));
    if hidden {
        match atlases.hidden_center(group, locked) {
            Some(center) => layer(art, center, size),
            None => fallback_art(art, "?", size.x),
        }
        if !locked
            && let Some(mark) = atlases.undiscovered_mark(matches!(
                group,
                ContentGroup::Jokers | ContentGroup::Editions
            ))
        {
            layer(art, mark, size);
        }
        return;
    }
    let Some(sprite) = collection_sprite(atlases, group, id) else {
        fallback_art(art, id, size.x);
        return;
    };
    match group {
        ContentGroup::Editions => {
            layer(art, sprite.clone(), size);
            if let Some(edition) = edition_label(id) {
                crate::components::edition::overlay(art, edition, &sprite, 0);
            }
        }
        ContentGroup::Seals => {
            layer(art, sprite, size);
            if let Some(stamp) = atlases.seal(seal_name(id)) {
                layer(art, stamp, size);
            }
        }
        _ => layer(art, sprite, size),
    }
}

/// `Card:set_sprites` hides undiscovered Jokers, editions, consumables, vouchers and
/// boosters; enhancements and seals always show.
pub(super) fn hides_undiscovered(group: ContentGroup) -> bool {
    !matches!(group, ContentGroup::Enhancements | ContentGroup::Seals)
}

fn layer(art: &mut ChildSpawnerCommands, image: ImageNode, size: Vec2) {
    art.spawn((
        image,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: px(size.x),
            height: px(size.y),
            ..default()
        },
        Pickable::IGNORE,
    ));
}

/// The edition shader of a collection edition; `None` for the plain base.
pub(super) fn edition_label(id: &str) -> Option<&'static str> {
    match id {
        "e_foil" => Some("Foil"),
        "e_holo" => Some("Holographic"),
        "e_polychrome" => Some("Polychrome"),
        "e_negative" => Some("Negative"),
        _ => None,
    }
}

fn seal_name(id: &str) -> &'static str {
    match id {
        "gold_seal" => "Gold",
        "red_seal" => "Red",
        "blue_seal" => "Blue",
        _ => "Purple",
    }
}

pub(super) fn collection_tip(
    item: &Content,
    group: ContentGroup,
    discovered: bool,
    locked: bool,
) -> tooltip::Tip {
    let tip = if locked {
        // `generate_card_ui` with `card_type = 'Locked'`: the name "Locked" and the unlock
        // condition, without badges.
        tooltip::Tip::new("Locked", item.unlock.unwrap_or("Keep playing to unlock"))
    } else if discovered {
        discovered_tip(item, group)
    } else if group == ContentGroup::Blinds {
        tooltip::Tip::new("Not Discovered", "Defeat this blind to discover it.")
    } else {
        tooltip::Tip::new(
            "Not Discovered",
            "Find this card in an unseeded run to learn what it does.",
        )
    };
    tip.technical(format!("ID: {}", item.id.0))
        .technical(format!("Group: {group:?}"))
        .technical(format!("Cost: ${}", item.cost))
        .technical(format!("Supported: {}", item.supported))
        .technical(format!("Discovered: {discovered}"))
        .technical(format!("Locked: {locked}"))
}

/// The popup a discovered entry has in a run, as `Card:generate_UIBox_ability_table` builds
/// it: Jokers carry their rarity, consumables their set, vouchers and boosters their type;
/// an enhancement shows only its rule and its name as a badge; an edition is the plain
/// Joker with the "Edition" badge and its own; tags and blinds have no badge.
fn discovered_tip(item: &Content, group: ContentGroup) -> tooltip::Tip {
    let tip = tooltip::Tip::new(&item.name, &item.description);
    let set = match group {
        ContentGroup::Jokers => item.rarity,
        ContentGroup::Tarots => Some("Tarot"),
        ContentGroup::Planets => Some("Planet"),
        ContentGroup::Spectrals => Some("Spectral"),
        ContentGroup::Vouchers => Some("Voucher"),
        ContentGroup::Boosters => Some("Booster"),
        ContentGroup::Seals => Some(item.name.as_str()),
        ContentGroup::Enhancements => {
            return tooltip::Tip::new("", &item.description).badge(&item.name);
        }
        ContentGroup::Editions => {
            let tip = tip.badge("Edition");
            return match edition_label(item.id.0) {
                Some(edition) => tip.badge(edition),
                None => tip,
            };
        }
        ContentGroup::Tags | ContentGroup::Blinds => None,
    };
    let tip = match set {
        Some(set) => tip.badge(set),
        None => tip,
    };
    tip.info(crate::components::info::references(item.id.0))
}

pub(super) fn deck_tip(
    deck: DeckChoice,
    locked: bool,
    condition: Option<&'static str>,
) -> tooltip::Tip {
    let tip = if locked {
        tooltip::Tip::new("Locked", condition.unwrap_or("Keep playing to unlock"))
    } else {
        tooltip::Tip::new(deck.name(), deck.effect())
    };
    tip.technical(format!("ID: {}", deck.id()))
        .technical(format!("Locked: {locked}"))
}

pub(super) fn lock_overlay(ui: &mut ChildSpawnerCommands) {
    let metal = Color::srgb(0.86, 0.88, 0.85);
    ui.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            right: px(0),
            bottom: px(0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.03, 0.05, 0.06, 0.74)),
        ZIndex(2),
        Pickable::IGNORE,
    ))
    .with_children(|overlay| {
        overlay
            .spawn((
                Node {
                    width: px(22),
                    height: px(18),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                BackgroundColor(metal),
                BorderColor::all(Color::srgb(0.09, 0.11, 0.11)),
                ZIndex(3),
                Pickable::IGNORE,
            ))
            .with_children(|body| {
                body.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(2),
                        top: px(-12),
                        width: px(14),
                        height: px(14),
                        border: UiRect {
                            left: px(3),
                            right: px(3),
                            top: px(3),
                            bottom: px(0),
                        },
                        border_radius: BorderRadius {
                            top_left: px(7).into(),
                            top_right: px(7).into(),
                            ..default()
                        },
                        ..default()
                    },
                    BorderColor::all(metal),
                    Pickable::IGNORE,
                ));
            });
    });
}

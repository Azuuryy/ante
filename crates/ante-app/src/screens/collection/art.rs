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
        ContentGroup::Editions | ContentGroup::Seals => atlases.card_base(),
    }
}

/// Draws an entry like the original collection: editions on a plain card with their
/// shader, seals stamped on a plain card, blinds as chips and the rest as their sprite.
pub(super) fn collection_art(
    art: &mut ChildSpawnerCommands,
    atlases: &Atlases,
    group: ContentGroup,
    id: &str,
    sprite: ImageNode,
) {
    match group {
        ContentGroup::Blinds => atlas_image(art, sprite, 66.0, 66.0),
        ContentGroup::Editions => {
            let label = match id {
                "e_foil" => "Foil",
                "e_holo" => "Holographic",
                "e_polychrome" => "Polychrome",
                _ => "Negative",
            };
            atlas_image(art, sprite.clone(), 66.0, 88.0);
            crate::components::edition::overlay(art, label, &sprite, 0);
        }
        ContentGroup::Seals => {
            let seal = match id {
                "gold_seal" => "Gold",
                "red_seal" => "Red",
                "blue_seal" => "Blue",
                _ => "Purple",
            };
            atlas_image(art, sprite, 66.0, 88.0);
            if let Some(stamp) = atlases.seal(seal) {
                art.spawn((
                    stamp,
                    Node {
                        position_type: PositionType::Absolute,
                        width: px(66),
                        height: px(88),
                        ..default()
                    },
                    Pickable::IGNORE,
                ));
            }
        }
        _ => atlas_image(art, sprite, 66.0, 88.0),
    }
}

pub(super) fn collection_tip(
    item: &Content,
    group: ContentGroup,
    discovered: bool,
    locked: bool,
) -> tooltip::Tip {
    let tip = if locked {
        tooltip::Tip::new(&item.name, "Locked")
            .badge("Locked")
            .detail(format!(
                "Unlock: {}",
                item.unlock.unwrap_or("Keep playing to unlock")
            ))
    } else if discovered {
        tooltip::Tip::new(&item.name, &item.description)
            .badge(if item.supported {
                "Discovered"
            } else {
                "Unavailable"
            })
            .info(crate::components::info::references(item.id.0))
    } else {
        tooltip::Tip::new(&item.name, "Undiscovered")
            .detail("Find this item in a run to reveal it.")
    };
    tip.technical(format!("ID: {}", item.id.0))
        .technical(format!("Group: {group:?}"))
        .technical(format!("Cost: ${}", item.cost))
        .technical(format!("Supported: {}", item.supported))
        .technical(format!("Discovered: {discovered}"))
        .technical(format!("Locked: {locked}"))
}

pub(super) fn deck_tip(
    deck: DeckChoice,
    locked: bool,
    condition: Option<&'static str>,
) -> tooltip::Tip {
    let tip = if locked {
        tooltip::Tip::new(deck.name(), "Locked")
            .badge("Locked")
            .detail(format!(
                "Unlock: {}",
                condition.unwrap_or("Keep playing to unlock")
            ))
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

use super::*;

/// One `info_tip_from_rows` box in a JOKER_GREY frame: the title on grey, then the rule
/// on a white box.
pub(super) fn side_box(ui: &mut ChildSpawnerCommands, info: &Info) {
    ui.spawn((
        Node {
            width: percent(100),
            padding: UiRect::all(tiles(0.05)),
            ..default()
        },
        PixelBox::new(lighten(JOKER_GREY, 0.5)).emboss(0.05),
        Pickable::IGNORE,
    ))
    .with_children(|frame| {
        frame
            .spawn((
                Node {
                    flex_grow: 1.0,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                PixelBox::new(SIDE_PANEL),
                Pickable::IGNORE,
            ))
            .with_children(|panel| {
                panel
                    .spawn((
                        Node {
                            min_height: tiles(0.36),
                            padding: UiRect::all(tiles(0.03)),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|title| {
                        styled_text(title, info.title, text_px(DESC_SCALE), Color::WHITE, true);
                    });
                panel
                    .spawn((
                        Node {
                            width: percent(100),
                            min_height: tiles(0.4),
                            padding: UiRect::all(tiles(0.08)),
                            justify_content: JustifyContent::Center,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        PixelBox::new(PAPER),
                        Pickable::IGNORE,
                    ))
                    .with_children(|rule| {
                        effect_line(rule, info.body, TEXT_DARK, text_px(DESC_SCALE));
                    });
            });
    });
}

/// A text node with the game font size and an optional drop shadow.
pub(super) fn styled_text(
    ui: &mut ChildSpawnerCommands,
    text: impl Into<String>,
    size: f32,
    color: Color,
    shadow: bool,
) {
    let mut entity = ui.spawn((
        Text::new(text.into()),
        TextFont {
            font_size: bevy::text::FontSize::Px(size),
            ..default()
        },
        TextColor(color),
        Pickable::IGNORE,
    ));
    if shadow {
        entity.insert(text_shadow());
    }
}

/// The `playing_card` name line on one row at the description scale: the rank and "of" in
/// `G.C.L_BLACK`, the suit in its colour.
pub(super) fn playing_card_name(ui: &mut ChildSpawnerCommands, title: &str) {
    let size = text_px(DESC_SCALE);
    ui.spawn((
        Node {
            flex_wrap: FlexWrap::NoWrap,
            column_gap: px(size * 0.3),
            align_items: AlignItems::Center,
            ..default()
        },
        Pickable::IGNORE,
    ))
    .with_children(|row| {
        for word in title.split_whitespace() {
            let color = suit_color(&word.to_ascii_lowercase()).unwrap_or(TEXT_DARK);
            label(row, word, size, color);
        }
    });
}

/// `G.C.SUITS` colours by suit word.
pub(super) fn suit_color(word: &str) -> Option<Color> {
    Some(match word {
        "heart" | "hearts" => Color::srgb(0.92, 0.06, 0.19),
        "diamond" | "diamonds" => Color::srgb(0.74, 0.32, 0.04),
        "club" | "clubs" => Color::srgb(0.00, 0.40, 0.27),
        "spade" | "spades" => Color::srgb(0.35, 0.20, 0.54),
        _ => return None,
    })
}

/// Name scale of `localize{type = 'name'}`: `0.55 - 0.004 x length`.
pub(super) fn name_scale(name: &str) -> f32 {
    0.55 - 0.004 * name.chars().count() as f32
}

pub(super) fn main_panel(panel: &mut ChildSpawnerCommands, tip: &Tip) {
    let name_size = text_px(name_scale(&tip.title));
    if tip.paper_name {
        panel
            .spawn((
                Node {
                    padding: UiRect::axes(tiles(0.1), tiles(0.05)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                PixelBox::new(PAPER).emboss(0.05),
                Pickable::IGNORE,
            ))
            .with_children(|name| playing_card_name(name, &tip.title));
    } else if !tip.title.is_empty() {
        // An enhancement in the collection has no name line (`full_UI_table.name = true`).
        panel
            .spawn((
                Node {
                    padding: UiRect::all(tiles(0.05)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|name| styled_text(name, &tip.title, name_size, Color::WHITE, true));
    }
    if !tip.effect.is_empty() || tip.currently.is_some() {
        // `desc_from_rows`: a white box of at least 2 x 0.8 tiles.
        panel
            .spawn((
                Node {
                    width: percent(100),
                    min_width: tiles(2.0),
                    min_height: tiles(0.8),
                    padding: UiRect::all(tiles(0.07)),
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: tiles(0.03),
                    ..default()
                },
                PixelBox::new(PAPER).emboss(0.05),
                Pickable::IGNORE,
            ))
            .with_children(|effect| {
                for line in tip.effect.lines() {
                    effect_line(effect, line, TEXT_DARK, text_px(DESC_SCALE));
                }
                if let Some(currently) = &tip.currently {
                    effect_line(effect, currently, TEXT_INACTIVE, text_px(DESC_SCALE));
                }
            });
    }
    if !tip.badges.is_empty() {
        panel
            .spawn((
                Node {
                    width: percent(100),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: tiles(0.03),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|badges| {
                for (index, badge) in tip.badges.iter().enumerate() {
                    // `create_badge`; the card type badge is drawn at 1.2 times the size.
                    let scaling = if index == 0 && is_type_badge(badge) {
                        1.2
                    } else {
                        1.0
                    };
                    badges
                        .spawn((
                            Node {
                                min_width: tiles(2.0),
                                min_height: tiles(0.4 * scaling),
                                padding: UiRect::axes(tiles(0.06), tiles(0.03 * scaling)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            PixelBox::new(badge_color(badge)).emboss(0.05),
                            Pickable::IGNORE,
                        ))
                        .insert_if(DarkEditionBadge, || is_dark_edition_badge(badge))
                        .with_children(|pill| {
                            styled_text(pill, badge, text_px(0.33 * scaling), Color::WHITE, true);
                        });
                }
            });
    }
    for detail in &tip.details {
        label(panel, detail, text_px(0.28), MUTED);
    }
}

/// Card type badges (rarity, consumable set, voucher, booster, enhancement).
pub(super) fn is_dark_edition_badge(badge: &str) -> bool {
    matches!(badge, "Foil" | "Holographic" | "Polychrome" | "Negative")
}

pub(super) fn is_type_badge(badge: &str) -> bool {
    is_rarity_badge(badge)
        || matches!(
            badge,
            "Tarot" | "Planet" | "Spectral" | "Voucher" | "Booster" | "Edition" | "Debuffed"
        )
        || badge.ends_with(" Card")
}

pub(super) fn is_rarity_badge(badge: &str) -> bool {
    matches!(badge, "Common" | "Uncommon" | "Rare" | "Legendary")
}

pub(super) fn badge_color(badge: &str) -> Color {
    match badge {
        "Common" => RARITY_COMMON,
        "Uncommon" => RARITY_UNCOMMON,
        "Rare" => RARITY_RARE,
        "Legendary" => RARITY_LEGENDARY,
        "Tarot" => TAROT,
        "Planet" => PLANET,
        "Spectral" => SPECTRAL,
        "Voucher" => VOUCHER,
        "Booster" => BOOSTER,
        "Edition" => EDITION,
        "Foil" | "Holographic" | "Polychrome" | "Negative" => dark_edition(0.0),
        "Locked" => LOCKED,
        "Gold Seal" => crate::components::tiles::GOLD,
        "Red Seal" | "Debuffed" => crate::components::tiles::RED,
        "Blue Seal" => crate::components::tiles::BLUE,
        "Purple Seal" => PURPLE,
        badge if badge.ends_with(" Card") => ENHANCED,
        _ => crate::components::tiles::BLUE,
    }
}

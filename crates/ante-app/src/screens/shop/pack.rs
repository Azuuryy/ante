use super::*;

/// `choose` and `extra` of a booster (`G.P_CENTERS.p_*`): cards to pick out of the cards shown.
pub(crate) fn pack_counts(
    family: ante_core::contract::PackFamily,
    size: ante_core::contract::PackSize,
) -> (usize, usize) {
    use ante_core::contract::{PackFamily::*, PackSize::*};
    let extra = match (family, size) {
        (Spectral | Buffoon, Normal) => 2,
        (Spectral | Buffoon, _) => 4,
        (_, Normal) => 3,
        (_, _) => 5,
    };
    (if size == Mega { 2 } else { 1 }, extra)
}

/// Width of the pack card area of `create_UIBox_*_pack`: `n × CARD_W` for Arcana and
/// Spectral, `× 1.1` for Standard and Buffoon, `× 1.1 + 0.5` for Celestial. The cards
/// share one row and close up instead of wrapping.
pub(super) fn pack_area_width(
    family: Option<ante_core::contract::PackFamily>,
    count: usize,
) -> f32 {
    use ante_core::contract::PackFamily::*;
    let card = menu_ui::CARD_W * menu_ui::TILE;
    let n = count.max(1) as f32;
    match family {
        Some(Arcana | Spectral) => n * card,
        Some(Celestial) => n * card * 1.1 + 0.5 * menu_ui::TILE,
        Some(Standard | Buffoon) | None => n * card * 1.1,
    }
}

/// Left edge of pack card `index`: the `consumeable` alignment spreads the cards over the
/// area by the unscaled card width and centres a single card.
pub(super) fn pack_card_left(index: usize, count: usize, area: f32) -> f32 {
    let card = menu_ui::CARD_W * menu_ui::TILE;
    let slot = if count > 1 {
        (area - card) * index as f32 / (count - 1) as f32
    } else {
        (area - card) / 2.0
    };
    slot + 0.5 * (card - menu_ui::CARD_WIDTH)
}

/// Localization key parts of a booster: `p_<family>_<size>`.
pub(super) fn pack_key(
    family: ante_core::contract::PackFamily,
    size: ante_core::contract::PackSize,
) -> (&'static str, &'static str) {
    use ante_core::contract::{PackFamily::*, PackSize::*};
    (
        match family {
            Arcana => "arcana",
            Celestial => "celestial",
            Spectral => "spectral",
            Standard => "standard",
            Buffoon => "buffoon",
        },
        match size {
            Normal => "normal",
            Jumbo => "jumbo",
            Mega => "mega",
        },
    )
}

/// The booster popup text, read from the local localization when available.
pub(crate) fn pack_description(
    text: &UiText,
    family: ante_core::contract::PackFamily,
    size: ante_core::contract::PackSize,
) -> String {
    let (choose, extra) = pack_counts(family, size);
    let (family, size) = pack_key(family, size);
    text.pack(family, size, choose, extra)
}

/// Header of the open pack box.
pub(super) fn pack_title(family: Option<ante_core::contract::PackFamily>) -> &'static str {
    use ante_core::contract::PackFamily::*;
    match family {
        Some(Arcana) => "Arcana Pack",
        Some(Celestial) => "Celestial Pack",
        Some(Spectral) => "Spectral Pack",
        Some(Standard) => "Standard Pack",
        Some(Buffoon) => "Buffoon Pack",
        None => "Booster Pack",
    }
}

/// `create_UIBox_arcana_pack` and its siblings: the pack box rises from below the hand; for
/// Arcana and Spectral packs the drawn hand stays under it, as `draw_from_deck_to_hand` deals
/// it, and target cards are picked in that hand.
pub(crate) fn pack_board(ui: &mut ChildSpawnerCommands, view: &RunView, atlases: &Atlases) {
    ui.spawn((
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: menu_ui::tiles(0.3),
            ..default()
        },
        Pickable::IGNORE,
    ))
    .with_children(|area| {
        area.spawn((
            Node {
                padding: UiRect::all(menu_ui::tiles(0.1)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: menu_ui::tiles(0.1),
                ..default()
            },
            PixelBox::new(menu_ui::TRANSPARENT_DARK),
            UiTransform::IDENTITY,
            crate::components::sparkles::PackRise,
        ))
        .with_children(|pack| {
            let size = menu_ui::card_px();
            let count = view.pack_choices.len();
            let area = pack_area_width(view.open_pack, count);
            pack.spawn(Node {
                width: px(area),
                height: px(size.y),
                flex_shrink: 0.0,
                ..default()
            })
            .with_children(|choices| {
                for (index, item) in view.pack_choices.iter().enumerate() {
                    let mut card = choices.spawn(Node {
                        position_type: PositionType::Absolute,
                        left: px(pack_card_left(index, count, area)),
                        top: px(0),
                        ..default()
                    });
                    card_hover(&mut card);
                    if item.enabled {
                        click_target(
                            &mut card,
                            Click::Play(Command::PickPack {
                                index,
                                revision: view.revision,
                            }),
                        );
                    }
                    tooltip::attach(
                        &mut card,
                        tooltip::art(
                            art_tip(item.art, &item.name, &item.description)
                                .technical(format!("Pack choice {index}"))
                                .technical(format!("{:?}", item.art))
                                .technical(format!("Enabled: {}", item.enabled)),
                            shop_art(atlases, item.art).as_ref(),
                        ),
                    );
                    card.with_children(|art| {
                        if matches!(item.art, ante_core::contract::ShopArt::Card { .. }) {
                            playing_art(art, atlases, item.art, index, size.x);
                        } else if let Some(image) = shop_art(atlases, item.art) {
                            atlas_image(art, image.clone(), size.x, size.y);
                            if let ante_core::contract::ShopArt::Consumable(id) = item.art {
                                crate::components::edition::overlay(
                                    art,
                                    crate::components::edition::consumable_look(id),
                                    &image,
                                    index,
                                );
                            }
                        } else {
                            fallback_art(art, &item.name, size.x);
                        }
                    });
                }
            });
            pack.spawn(Node {
                column_gap: menu_ui::tiles(0.3),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|footer| {
                footer
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|title| {
                        shadowed_label(
                            title,
                            pack_title(view.open_pack),
                            menu_ui::text_px(0.6),
                            INK,
                        );
                        let hint = if view.phase == Phase::Target {
                            "Select target cards".to_owned()
                        } else {
                            format!("Choose {}", view.pack_picks)
                        };
                        shadowed_label(title, hint, menu_ui::text_px(0.4), INK);
                    });
                if view.phase == Phase::Target {
                    menu_ui::card_button(
                        footer,
                        &[&[("Use", 0.45)]],
                        menu_ui::RED,
                        (1.6, 0.7),
                        UiRect::all(menu_ui::tiles(0.1)),
                        view.can_apply_target
                            .then_some(Click::Play(Command::ApplyTarget)),
                    );
                }
                if view.can_skip_pack {
                    menu_ui::card_button(
                        footer,
                        &[&[("Skip", 0.45)]],
                        menu_ui::GREY,
                        (1.6, 0.7),
                        UiRect::all(menu_ui::tiles(0.1)),
                        Some(Click::Play(Command::SkipPack)),
                    );
                }
            });
        });
        if !view.cards.is_empty() {
            crate::screens::hand::hand_rows(area, view, atlases, false);
        }
    });
}

#[cfg(test)]
mod tests;

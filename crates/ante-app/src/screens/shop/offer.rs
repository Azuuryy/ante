use super::*;

/// A light shop box (`G.C.L_BLACK`) with padding and minimum width in tiles.
pub(super) fn shop_box(
    ui: &mut ChildSpawnerCommands,
    padding: f32,
    min_width: f32,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    ui.spawn((
        Node {
            padding: UiRect::all(menu_ui::tiles(padding)),
            min_width: menu_ui::tiles(min_width),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        PixelBox::new(menu_ui::L_BLACK).emboss(0.05),
    ))
    .with_children(content);
}

/// Text turned a quarter left, like the voucher box's label.
pub(super) fn vertical_label(ui: &mut ChildSpawnerCommands, ante: usize, height: f32) {
    ui.spawn((
        Node {
            width: menu_ui::tiles(0.4),
            height: menu_ui::tiles(height),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Pickable::IGNORE,
    ))
    .with_children(|column| {
        column
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: menu_ui::tiles(height),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                UiTransform::from_rotation(Rot2::degrees(-90.0)),
                Pickable::IGNORE,
            ))
            .with_children(|text_box| {
                text_box.spawn((
                    Text::new(""),
                    TextFont {
                        font_size: bevy::text::FontSize::Px(menu_ui::text_px(0.45)),
                        ..default()
                    },
                    TextColor(menu_ui::L_BLACK),
                    LocalText::with_var("ante_x_voucher", ante),
                    Pickable::IGNORE,
                ));
            });
    });
}

/// A card area of the shop: `size` in tiles, offers spread over `limit` slots and drawn at
/// `scale` card sizes.
pub(super) fn offer_area(
    ui: &mut ChildSpawnerCommands,
    offers: &[Offer],
    (selected, revision, atlases): (Option<Command>, u64, &Atlases),
    (width, height): (f32, f32),
    (limit, scale): (usize, f32),
) {
    let card = menu_ui::card_px() * scale;
    let area = Vec2::new(menu_ui::tile_px(width), menu_ui::tile_px(height));
    ui.spawn(Node {
        width: px(area.x),
        height: px(area.y),
        flex_shrink: 0.0,
        ..default()
    })
    .with_children(|zone| {
        for (index, offer) in offers.iter().enumerate() {
            let chosen = selected == Some(offer.action);
            let mut item = zone.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(menu_ui::spread_x(
                        index,
                        offers.len(),
                        limit,
                        area.x,
                        card.x,
                    )),
                    top: px((area.y - card.y) / 2.0),
                    width: px(card.x),
                    height: px(card.y),
                    ..default()
                },
                ZIndex(if chosen { SELECTED_OFFER_Z } else { 0 }),
            ));
            if !chosen {
                card_hover(&mut item);
            }
            click_target(
                &mut item,
                Click::SelectOffer {
                    action: offer.action,
                    revision,
                },
            );
            tooltip::attach_anchored(
                &mut item,
                tooltip::art(offer.tip.clone(), offer.image.as_ref()),
                tooltip::TipAnchor::Left,
            );
            item.with_children(|art| {
                price_tag(art, offer.price, card.y);
                if chosen {
                    offer_buttons(art, offer, card);
                }
                if let Some(image) = &offer.image {
                    if let Some(face) = offer.playing_card {
                        playing_art(art, atlases, face, offer.edition.1, card.x);
                    } else {
                        atlas_image(art, image.clone(), card.x, card.y);
                        crate::components::edition::overlay(
                            art,
                            &offer.edition.0,
                            image,
                            offer.edition.1,
                        );
                    }
                } else {
                    fallback_art(art, &offer.tip.title, card.x);
                }
            });
        }
    });
}

/// `create_shop_card_ui`'s price tag: a dark tab behind the card's top edge, its price in a
/// lighter box.
pub(super) fn price_tag(ui: &mut ChildSpawnerCommands, price: usize, card_height: f32) {
    ui.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            bottom: px(card_height - menu_ui::tile_px(PRICE_TAG_OVERLAP)),
            justify_content: JustifyContent::Center,
            ..default()
        },
        ZIndex(-1),
        Pickable::IGNORE,
    ))
    .with_children(|anchor| {
        anchor
            .spawn((
                Node {
                    min_width: menu_ui::tiles(0.6),
                    min_height: menu_ui::tiles(1.0),
                    padding: UiRect::all(menu_ui::tiles(0.05)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                PixelBox::new(menu_ui::darken(menu_ui::BLACK, 0.2)),
                Pickable::IGNORE,
            ))
            .with_children(|tag| {
                tag.spawn((
                    Node {
                        min_width: menu_ui::tiles(1.0),
                        min_height: menu_ui::tiles(0.55),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    PixelBox::new(menu_ui::lighten(menu_ui::BLACK, 0.1)),
                    Pickable::IGNORE,
                ))
                .with_children(|inner| {
                    label(
                        inner,
                        format!("${price}"),
                        menu_ui::text_px(0.5),
                        menu_ui::MONEY,
                    );
                });
            });
    });
}

/// The buttons of a selected offer, drawn behind the card: the main one under it ('bm',
/// offset -0.3) and the consumable's "Buy & Use" to its right ('cr', offset -0.3).
pub(super) fn offer_buttons(ui: &mut ChildSpawnerCommands, offer: &Offer, card: Vec2) {
    let overlap = menu_ui::tile_px(0.3);
    let (text, color) = match offer.verb {
        Verb::Buy => ("BUY", menu_ui::ORANGE),
        Verb::Redeem => ("REDEEM", menu_ui::GREEN),
        Verb::Open => ("OPEN", menu_ui::GREEN),
    };
    ui.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(card.y - overlap),
            justify_content: JustifyContent::Center,
            ..default()
        },
        ZIndex(-1),
    ))
    .observe(|mut event: On<PointerPress>| event.propagate(false))
    .with_children(|anchor| {
        menu_ui::card_button(
            anchor,
            &[&[(text, 0.5)]],
            color,
            (1.1, 0.94),
            UiRect {
                top: px(overlap),
                ..UiRect::all(menu_ui::tiles(0.05))
            },
            offer.enabled.then_some(Click::Play(offer.action)),
        );
    });
    if let Some(action) = offer.buy_and_use {
        ui.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(card.x - overlap),
                top: px(0),
                bottom: px(0),
                align_items: AlignItems::Center,
                ..default()
            },
            ZIndex(-1),
        ))
        .observe(|mut event: On<PointerPress>| event.propagate(false))
        .with_children(|anchor| {
            menu_ui::card_button(
                anchor,
                &[&[("BUY", 0.5)], &[("& USE", 0.3)]],
                menu_ui::RED,
                (1.1, 1.1),
                UiRect {
                    left: px(overlap),
                    ..UiRect::all(menu_ui::tiles(0.05))
                },
                Some(Click::Play(action)),
            );
        });
    }
}

/// The popup of a shop or pack item: a playing card gets the playing card popup.
pub(super) fn art_tip(
    art: ante_core::contract::ShopArt,
    name: &str,
    description: &str,
) -> tooltip::Tip {
    match art {
        ante_core::contract::ShopArt::Card { rank, suit, look } => tooltip::playing_card(
            rank,
            suit,
            look.enhancement.unwrap_or_default(),
            look.edition,
            look.seal.unwrap_or_default(),
            false,
        ),
        _ => tooltip::Tip::new(name, description),
    }
}

/// A playing card offer or pack choice with its enhancement, edition and seal.
pub(super) fn playing_art(
    ui: &mut ChildSpawnerCommands,
    atlases: &Atlases,
    art: ante_core::contract::ShopArt,
    seed: usize,
    width: f32,
) {
    if let ante_core::contract::ShopArt::Card { rank, suit, look } = art {
        let face = crate::components::widgets::PlayingFace {
            rank,
            suit,
            enhancement: look.enhancement.unwrap_or_default(),
            edition: look.edition,
            seal: look.seal.unwrap_or_default(),
            debuffed: false,
            seed,
        };
        if !crate::components::widgets::playing_card_face(ui, atlases, &face, width) {
            fallback_art(ui, &format!("{rank} of {suit}"), width);
        }
    }
}

pub(crate) fn shop_art(atlases: &Atlases, art: ante_core::contract::ShopArt) -> Option<ImageNode> {
    use ante_core::contract::ShopArt::*;
    match art {
        Card { rank, suit, .. } => atlases.card(rank, suit),
        Joker(id) => atlases.joker(id),
        Consumable(id) => atlases.consumable(id),
        Pack { family, size } => atlases.booster(family, size),
    }
}

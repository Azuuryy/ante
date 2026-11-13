use crate::screens::run::LocalText;
use crate::{
    app::session::Click,
    components::{
        cards::{consumable_tooltip, joker_tooltip},
        pixel_rect::{DynRole, PixelBox},
        theme::INK,
        tiles as menu_ui, tooltip,
        widgets::{
            atlas_image, card_hover, click_target, fallback_art, label, shadowed_label, wrap_row,
        },
    },
    services::assets::{catalog::Atlases, ui_text::UiText},
};
use ante_core::contract::{Command, Phase, RunView};
use bevy::picking::prelude::Pickable;
use bevy::prelude::*;

pub(crate) fn shop_board(
    ui: &mut ChildSpawnerCommands,
    view: &RunView,
    atlases: &Atlases,
    selected: Option<Command>,
    text: &UiText,
) {
    use ante_core::contract::ShopArt;
    let cards = view
        .offers
        .iter()
        .map(|card| Offer {
            action: Command::BuyJoker(card.id),
            enabled: card.enabled,
            price: card.price,
            tip: joker_tooltip(card),
            image: atlases.joker(card.kind),
            playing_card: None,
            verb: Verb::Buy,
            buy_and_use: None,
            edition: (card.edition.clone(), card.id.0),
        })
        .chain(view.consumable_offers.iter().map(|card| {
            Offer {
                action: Command::BuyConsumable {
                    index: card.index,
                    revision: view.revision,
                },
                enabled: card.can_buy,
                price: card.price,
                tip: consumable_tooltip(card),
                image: atlases.consumable(card.kind),
                playing_card: None,
                verb: Verb::Buy,
                buy_and_use: card
                    .can_buy_and_use
                    .then_some(Command::BuyAndUseConsumable {
                        index: card.index,
                        revision: view.revision,
                    }),
                edition: (
                    crate::components::edition::consumable_look(card.kind).to_owned(),
                    card.index,
                ),
            }
        }))
        .chain(view.shop_cards.iter().map(|(id, card)| {
            Offer {
                action: Command::BuyPlayingCard {
                    id: *id,
                    revision: view.revision,
                },
                enabled: card.enabled,
                price: card.price,
                tip: art_tip(card.art, &card.name, &card.description)
                    .technical(format!("Shop card #{}", id.0))
                    .technical(format!("{:?}", card.art)),
                image: shop_art(atlases, card.art),
                playing_card: Some(card.art),
                verb: Verb::Buy,
                buy_and_use: None,
                edition: (String::new(), id.0),
            }
        }))
        .collect::<Vec<_>>();
    let vouchers = view
        .voucher_offers
        .iter()
        .map(|(item, enabled)| Offer {
            action: Command::RedeemVoucher {
                voucher: item.id,
                revision: view.revision,
            },
            enabled: *enabled,
            price: item.cost,
            tip: tooltip::Tip::new(&item.name, &item.description)
                .badge("Voucher")
                .technical(format!("{} · ${}", item.id.0, item.cost))
                .technical(format!("Supported: {}", item.supported)),
            image: atlases.voucher(item.id.0),
            playing_card: None,
            verb: Verb::Redeem,
            buy_and_use: None,
            edition: ("Voucher".to_owned(), 0),
        })
        .collect::<Vec<_>>();
    let packs = view
        .packs
        .iter()
        .enumerate()
        .map(|(index, pack)| Offer {
            action: Command::BuyPack {
                index,
                revision: view.revision,
            },
            enabled: pack.enabled,
            price: pack.price,
            tip: tooltip::Tip::new(
                &pack.name,
                match pack.art {
                    ShopArt::Pack { family, size } => pack_description(text, family, size),
                    _ => String::new(),
                },
            )
            .badge("Booster")
            .technical(format!("Pack index {index} · {:?}", pack.art)),
            image: match pack.art {
                ShopArt::Pack { family, size } => atlases.booster(family, size),
                _ => None,
            },
            verb: Verb::Open,
            buy_and_use: None,
            playing_card: None,
            edition: ("Booster".to_owned(), index),
        })
        .collect::<Vec<_>>();
    let joker_max = cards.len().max(SHOP_JOKER_SLOTS);
    // `UIBox_dyn_container`: a faint rim, the DYN_UI.MAIN outline (0.05, like the HUD) and a
    // BOSS_DARK body at least 30 units tall, so the panel runs below the window; inside, the
    // embossed BOSS_MAIN column.
    ui.spawn((
        Node {
            padding: UiRect::all(menu_ui::tiles(0.03)),
            flex_shrink: 0.0,
            ..default()
        },
        PixelBox::new(menu_ui::TRANSPARENT_DARK),
    ))
    .with_children(|rim| {
        rim.spawn((
            Node {
                padding: UiRect::all(menu_ui::tiles(0.05)),
                ..default()
            },
            PixelBox::new(menu_ui::BLACK),
            DynRole::Main,
        ))
        .with_children(|outline| {
            outline
                .spawn((
                    Node {
                        min_height: menu_ui::tiles(30.0),
                        padding: UiRect::all(menu_ui::tiles(0.1)),
                        align_items: AlignItems::Start,
                        ..default()
                    },
                    PixelBox::new(menu_ui::BLACK),
                    DynRole::BossDark,
                ))
                .with_children(|body| {
                    body.spawn((
                        Node {
                            padding: UiRect::all(menu_ui::tiles(0.1)),
                            row_gap: menu_ui::tiles(SHOP_ROW_GAP),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        PixelBox::new(menu_ui::BLACK).emboss(0.05),
                        DynRole::BossMain,
                    ))
                    .with_children(|board| {
                        board
                            .spawn(Node {
                                column_gap: menu_ui::tiles(0.2),
                                align_items: AlignItems::Center,
                                ..default()
                            })
                            .with_children(|top| {
                                top.spawn(Node {
                                    row_gap: menu_ui::tiles(0.1),
                                    flex_direction: FlexDirection::Column,
                                    ..default()
                                })
                                .with_children(|actions| {
                                    menu_ui::card_button(
                                        actions,
                                        &[&[("Next", 0.4)], &[("Round", 0.4)]],
                                        menu_ui::RED,
                                        (2.8, 1.5),
                                        UiRect::all(menu_ui::tiles(0.1)),
                                        Some(Click::Play(Command::LeaveShop)),
                                    );
                                    let cost = view.reroll_cost.to_string();
                                    menu_ui::card_button(
                                        actions,
                                        &[&[("Reroll", 0.4)], &[("$", 0.7), (cost.as_str(), 0.75)]],
                                        menu_ui::GREEN,
                                        (2.8, 1.6),
                                        UiRect::all(menu_ui::tiles(0.1)),
                                        view.can_reroll.then_some(Click::Play(Command::Reroll)),
                                    );
                                });
                                shop_box(top, 0.2, 8.2, |zone| {
                                    offer_area(
                                        zone,
                                        &cards,
                                        (selected, view.revision, atlases),
                                        (
                                            joker_max as f32 * 1.02 * menu_ui::CARD_W,
                                            1.05 * menu_ui::CARD_H,
                                        ),
                                        (joker_max, 1.0),
                                    );
                                });
                            });
                        board
                            .spawn(Node {
                                column_gap: menu_ui::tiles(0.2),
                                align_items: AlignItems::Stretch,
                                ..default()
                            })
                            .with_children(|bottom| {
                                shop_box(bottom, 0.15, 0.0, |outer| {
                                    outer
                                        .spawn((
                                            Node {
                                                padding: UiRect::all(menu_ui::tiles(0.2)),
                                                column_gap: menu_ui::tiles(0.1),
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            PixelBox::new(menu_ui::BLACK),
                                        ))
                                        .with_children(|inner| {
                                            vertical_label(
                                                inner,
                                                view.ante,
                                                1.05 * menu_ui::CARD_H,
                                            );
                                            if vouchers.is_empty() {
                                                restock_note(
                                                    inner,
                                                    (2.1 * menu_ui::CARD_W, 1.05 * menu_ui::CARD_H),
                                                );
                                            } else {
                                                offer_area(
                                                    inner,
                                                    &vouchers,
                                                    (selected, view.revision, atlases),
                                                    (2.1 * menu_ui::CARD_W, 1.05 * menu_ui::CARD_H),
                                                    (1, 1.0),
                                                );
                                            }
                                        });
                                });
                                shop_box(bottom, 0.15, 0.0, |zone| {
                                    offer_area(
                                        zone,
                                        &packs,
                                        (selected, view.revision, atlases),
                                        (2.4 * menu_ui::CARD_W, 1.15 * menu_ui::CARD_H),
                                        (2, BOOSTER_SCALE),
                                    );
                                });
                            });
                    });
                });
        });
    });
}

/// "DEFEAT / BOSS BLIND / TO RESTOCK" in an empty voucher area.
fn restock_note(ui: &mut ChildSpawnerCommands, (width, height): (f32, f32)) {
    ui.spawn((
        Node {
            width: menu_ui::tiles(width),
            height: menu_ui::tiles(height),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Pickable::IGNORE,
    ))
    .with_children(|note| {
        for (text, scale) in [("DEFEAT", 0.6), ("BOSS BLIND", 0.4), ("TO RESTOCK", 0.4)] {
            label(note, text, menu_ui::text_px(scale), INK);
        }
    });
}

/// Space between the Joker row and the voucher and booster row, in tiles.
const SHOP_ROW_GAP: f32 = 0.5;
/// How far the price tag's dark tab goes down behind the card, in tiles.
const PRICE_TAG_OVERLAP: f32 = 0.15;
/// Shop Jokers per row before vouchers like Overstock add slots.
const SHOP_JOKER_SLOTS: usize = 2;
/// Booster cards are created at 1.27 card sizes.
const BOOSTER_SCALE: f32 = 1.27;
/// The selected offer and its buttons draw over the other offers.
const SELECTED_OFFER_Z: i32 = 10;

/// What a selected offer's main button does.
#[derive(Clone, Copy)]
enum Verb {
    Buy,
    Redeem,
    Open,
}

struct Offer {
    action: Command,
    enabled: bool,
    price: usize,
    tip: tooltip::Tip,
    image: Option<ImageNode>,
    /// The art of a playing card offer, drawn with its enhancement, edition and seal.
    playing_card: Option<ante_core::contract::ShopArt>,
    verb: Verb,
    /// The consumable's "Buy & Use", when the original would show it.
    buy_and_use: Option<Command>,
    /// Engine edition label and per-card seed for the edition shader overlay.
    edition: (String, usize),
}

/// A light shop box (`G.C.L_BLACK`) with padding and minimum width in tiles.
fn shop_box(
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
fn vertical_label(ui: &mut ChildSpawnerCommands, ante: usize, height: f32) {
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
fn offer_area(
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
fn price_tag(ui: &mut ChildSpawnerCommands, price: usize, card_height: f32) {
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
fn offer_buttons(ui: &mut ChildSpawnerCommands, offer: &Offer, card: Vec2) {
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
fn art_tip(art: ante_core::contract::ShopArt, name: &str, description: &str) -> tooltip::Tip {
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
fn playing_art(
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

/// Localization key parts of a booster: `p_<family>_<size>`.
fn pack_key(
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
fn pack_title(family: Option<ante_core::contract::PackFamily>) -> &'static str {
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
            wrap_row(pack, |choices| {
                for (index, item) in view.pack_choices.iter().enumerate() {
                    let mut card = choices.spawn(Node::default());
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

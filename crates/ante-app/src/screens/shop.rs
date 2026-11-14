use crate::screens::run::LocalText;
use crate::{
    app::session::Click,
    components::{
        cards::{consumable_tooltip, joker_tooltip},
        pixel_rect::{DynRole, PixelBox},
        theme::INK,
        tiles as menu_ui, tooltip,
        widgets::{atlas_image, card_hover, click_target, fallback_art, label, shadowed_label},
    },
    services::assets::{catalog::Atlases, ui_text::UiText},
};
use ante_core::contract::{Command, Phase, RunView};
use bevy::picking::prelude::Pickable;
use bevy::prelude::*;

mod offer;
mod pack;

use offer::*;
pub(crate) use pack::*;

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

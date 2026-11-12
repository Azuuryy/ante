use crate::{
    app::session::Click,
    components::{
        cards::{consumable_tooltip, joker_tooltip},
        theme::{INK, MUTED},
        tiles as menu_ui, tooltip,
        widgets::{
            atlas_image, button, card_face, card_hover, click_target, fallback_art, label, row,
        },
    },
    services::assets::catalog::Atlases,
};
use ante_core::contract::{Command as MvpAction, Phase, RunView};
use bevy::prelude::*;

pub(crate) fn shop_board(
    ui: &mut ChildSpawnerCommands,
    view: &RunView,
    atlases: &Atlases,
    selected: Option<MvpAction>,
) {
    use ante_core::contract::ShopArt;
    let cards = view
        .offers
        .iter()
        .map(|card| Offer {
            action: MvpAction::BuyJoker(card.id),
            enabled: card.enabled,
            price: card.price,
            tip: joker_tooltip(card),
            image: atlases.joker(card.kind),
            playing_card: false,
            verb: "Buy",
            edition: (card.edition.clone(), card.id.0),
        })
        .chain(view.consumable_offers.iter().map(|card| Offer {
            action: MvpAction::BuyConsumable {
                index: card.index,
                revision: view.revision,
            },
            enabled: card.can_buy,
            price: card.price,
            tip: consumable_tooltip(card),
            image: atlases.consumable(card.kind),
            playing_card: false,
            verb: "Buy",
            edition: (String::new(), card.index),
        }))
        .chain(view.shop_cards.iter().map(|(id, card)| {
            Offer {
                action: MvpAction::BuyPlayingCard {
                    id: *id,
                    revision: view.revision,
                },
                enabled: card.enabled,
                price: card.price,
                tip: tooltip::Tip::new(
                    &card.name,
                    if card.description.is_empty() {
                        "Playing card"
                    } else {
                        card.description.as_str()
                    },
                )
                .technical(format!("Shop card #{}", id.0))
                .technical(format!("{:?}", card.art)),
                image: shop_art(atlases, card.art),
                playing_card: true,
                verb: "Buy",
                edition: (String::new(), id.0),
            }
        }))
        .collect::<Vec<_>>();
    let vouchers = view
        .voucher_offers
        .iter()
        .map(|(item, enabled)| Offer {
            action: MvpAction::RedeemVoucher {
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
            playing_card: false,
            verb: "Redeem",
            edition: ("Voucher".to_owned(), 0),
        })
        .collect::<Vec<_>>();
    let packs = view
        .packs
        .iter()
        .enumerate()
        .map(|(index, pack)| Offer {
            action: MvpAction::BuyPack {
                index,
                revision: view.revision,
            },
            enabled: pack.enabled,
            price: pack.price,
            tip: tooltip::Tip::new(&pack.name, "Open the pack to choose its contents.")
                .badge("Booster")
                .technical(format!("Pack index {index} · {:?}", pack.art)),
            image: match pack.art {
                ShopArt::Pack { family, size } => atlases.booster(family, size),
                _ => None,
            },
            verb: "Open",
            playing_card: false,
            edition: ("Booster".to_owned(), index),
        })
        .collect::<Vec<_>>();
    ui.spawn((
        Node {
            width: percent(100),
            padding: UiRect::all(px(12)),
            row_gap: px(9),
            flex_direction: FlexDirection::Column,
            border_radius: BorderRadius::all(px(14)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.10, 0.15, 0.16)),
    ))
    .with_children(|board| {
        board
            .spawn(Node {
                column_gap: px(10),
                ..default()
            })
            .with_children(|top| {
                top.spawn(Node {
                    width: px(161),
                    flex_shrink: 0.0,
                    row_gap: px(8),
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|actions| {
                    menu_ui::tile_button(
                        actions,
                        "Next Round",
                        menu_ui::RED,
                        (2.2, 1.22),
                        0.46,
                        Click::Play(MvpAction::LeaveShop),
                    );
                    menu_ui::tile_optional(
                        actions,
                        &format!("Reroll ${}", view.reroll_cost),
                        menu_ui::PALE_GREEN,
                        (2.2, 1.22),
                        0.44,
                        view.can_reroll.then_some(Click::Play(MvpAction::Reroll)),
                        false,
                    );
                });
                offer_zone(
                    top,
                    "Cards",
                    &cards,
                    selected,
                    view.revision,
                    (460.0, atlases),
                );
            });
        board
            .spawn(Node {
                column_gap: px(10),
                ..default()
            })
            .with_children(|bottom| {
                offer_zone(
                    bottom,
                    "Vouchers",
                    &vouchers,
                    selected,
                    view.revision,
                    (300.0, atlases),
                );
                offer_zone(
                    bottom,
                    "Boosters",
                    &packs,
                    selected,
                    view.revision,
                    (300.0, atlases),
                );
            });
    });
}

struct Offer {
    action: MvpAction,
    enabled: bool,
    price: usize,
    tip: tooltip::Tip,
    image: Option<ImageNode>,
    playing_card: bool,
    verb: &'static str,
    /// Engine edition label and per-card seed for the edition shader overlay.
    edition: (String, usize),
}

fn offer_zone(
    ui: &mut ChildSpawnerCommands,
    name: &str,
    offers: &[Offer],
    selected: Option<MvpAction>,
    revision: u64,
    (available, atlases): (f32, &Atlases),
) {
    let width = ((available - 8.0) / offers.len().max(1) as f32 - 5.0).clamp(38.0, 116.0);
    ui.spawn((
        Node {
            min_width: px(0),
            flex_basis: px(0),
            flex_grow: 1.0,
            min_height: px(211),
            padding: UiRect::all(px(7)),
            column_gap: px(5),
            row_gap: px(3),
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(9)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.19, 0.26, 0.27)),
    ))
    .with_children(|zone| {
        if offers.is_empty() {
            label(zone, name, 21.0, MUTED);
        }
        for offer in offers {
            zone.spawn(Node {
                width: px(width),
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .with_children(|item| {
                item.spawn((
                    Node {
                        padding: UiRect::axes(px(7), px(2)),
                        border_radius: BorderRadius::all(px(5)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.10, 0.14, 0.15)),
                ))
                .with_children(|price| {
                    label(
                        price,
                        format!("${}", offer.price),
                        21.0,
                        Color::srgb(0.97, 0.71, 0.27),
                    );
                });
                let mut hitbox = item.spawn(Node::default());
                card_hover(&mut hitbox);
                click_target(
                    &mut hitbox,
                    Click::SelectOffer {
                        action: offer.action,
                        revision,
                    },
                );
                tooltip::attach_anchored(
                    &mut hitbox,
                    tooltip::art(offer.tip.clone(), offer.image.as_ref()),
                    tooltip::TipAnchor::Left,
                );
                hitbox.with_children(|art| {
                    if let Some(image) = &offer.image {
                        if offer.playing_card {
                            card_face(art, atlases.card_base(), image.clone(), false, width);
                        } else {
                            atlas_image(art, image.clone(), width, width * 95.0 / 71.0);
                            crate::components::edition::overlay(
                                art,
                                &offer.edition.0,
                                image,
                                offer.edition.1,
                            );
                        }
                    } else {
                        fallback_art(art, &offer.tip.title, width);
                    }
                });
                if selected == Some(offer.action) {
                    button(
                        item,
                        offer.verb,
                        offer.enabled.then_some(Click::Play(offer.action)),
                    );
                }
            });
        }
    });
}

pub(crate) fn shop_art(atlases: &Atlases, art: ante_core::contract::ShopArt) -> Option<ImageNode> {
    use ante_core::contract::ShopArt::*;
    match art {
        Card { rank, suit } => atlases.card(rank, suit),
        Joker(id) => atlases.joker(id),
        Consumable(id) => atlases.consumable(id),
        Pack { family, size } => atlases.booster(family, size),
    }
}

pub(crate) fn pack_board(ui: &mut ChildSpawnerCommands, view: &RunView, atlases: &Atlases) {
    if view.phase == Phase::Target {
        label(ui, "Select target cards", 30.0, INK);
        row(ui, |cards| {
            for card in &view.cards {
                button(
                    cards,
                    format!(
                        "{} of {}{}",
                        card.rank,
                        card.suit,
                        if card.selected { " ✓" } else { "" }
                    ),
                    card.enabled
                        .then_some(Click::Play(MvpAction::ToggleCard(card.id))),
                );
            }
        });
        button(
            ui,
            "Apply",
            view.can_apply_target
                .then_some(Click::Play(MvpAction::ApplyTarget)),
        );
    } else {
        label(
            ui,
            format!("Booster Pack · Choose {}", view.pack_picks),
            30.0,
            INK,
        );
        row(ui, |choices| {
            for (index, item) in view.pack_choices.iter().enumerate() {
                let mut card = choices.spawn(Node::default());
                card_hover(&mut card);
                if item.enabled {
                    click_target(
                        &mut card,
                        Click::Play(MvpAction::PickPack {
                            index,
                            revision: view.revision,
                        }),
                    );
                }
                tooltip::attach(
                    &mut card,
                    tooltip::art(
                        tooltip::Tip::new(&item.name, &item.description)
                            .technical(format!("Pack choice {index}"))
                            .technical(format!("{:?}", item.art))
                            .technical(format!("Enabled: {}", item.enabled)),
                        shop_art(atlases, item.art).as_ref(),
                    ),
                );
                card.with_children(|art| {
                    if let Some(image) = shop_art(atlases, item.art) {
                        if matches!(item.art, ante_core::contract::ShopArt::Card { .. }) {
                            card_face(art, atlases.card_base(), image, false, 120.0);
                        } else {
                            atlas_image(art, image, 120.0, 160.0);
                        }
                    } else {
                        fallback_art(art, &item.name, 120.0);
                    }
                });
            }
        });
    }
    if view.can_skip_pack {
        button(ui, "Skip", Some(Click::Play(MvpAction::SkipPack)));
    }
}

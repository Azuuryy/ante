use super::{
    card_drag::{self, Lane},
    info, tiles as menu_ui, tooltip,
    widgets::{atlas_image, card_hover, fallback_art},
};
use crate::{
    app::{presentation::ScoredJoker, session::Click},
    services::assets::catalog::Atlases,
};
use ante_core::contract::{
    Command as MvpAction, ConsumableView, Content, JokerKey, JokerView, RunView,
};
use bevy::{
    picking::prelude::{PointerButton, PointerClick, PointerPress},
    prelude::*,
};

pub(crate) fn info_box(
    ui: &mut ChildSpawnerCommands,
    name: &str,
    tooltip_text: impl Into<tooltip::Tip>,
    sprite: Option<ImageNode>,
    action: Option<Click>,
    look: &str,
) {
    let mut entity = ui.spawn((
        Node {
            width: px(100),
            height: px(125),
            padding: UiRect::all(px(4)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.14, 0.19, 0.19)),
    ));
    card_hover(&mut entity);
    tooltip::attach(
        &mut entity,
        tooltip::art(tooltip_text.into(), sprite.as_ref()),
    );
    if let Some(action) = action {
        entity.observe(
            move |event: On<PointerClick>, mut clicks: MessageWriter<Click>| {
                if event.button == PointerButton::Primary {
                    clicks.write(action);
                }
            },
        );
    }
    entity.with_children(|card| {
        if let Some(sprite) = sprite {
            atlas_image(card, sprite.clone(), 95.0, 127.0);
            super::edition::overlay(card, look, &sprite, 0);
        } else {
            fallback_art(card, name, 95.0);
        }
    });
}

pub(crate) fn joker_tooltip(joker: &JokerView) -> tooltip::Tip {
    // Side boxes in the original order: what the effect mentions, then edition and stickers.
    let mut tip = tooltip::Tip::new(joker.name, &joker.description)
        .badge(&joker.rarity)
        .info(info::references(joker.kind))
        .info(info::edition(&joker.edition, false))
        .info(info::stickers(&joker.stickers));
    if !matches!(joker.edition.as_str(), "" | "Base" | "None") {
        tip = tip.badge(&joker.edition);
    }
    if let Some(current) = joker.current {
        tip = tip.currently(current);
    }
    if !joker.stickers.is_empty() {
        tip = tip.detail(&joker.stickers);
    }
    tip.technical(format!("{} · instance #{}", joker.kind, joker.id.0))
        .technical(format!("Engine state: {}", joker.state))
        .technical(format!(
            "Current: {}",
            joker
                .current
                .map_or_else(|| "none".to_owned(), |value| format!("{value:?}"))
        ))
        .technical(format!(
            "Price ${} · Sell ${} · can sell {}",
            joker.price, joker.sell_price, joker.can_sell
        ))
        .technical(format!(
            "Edition: {} · Stickers: {}",
            joker.edition,
            if joker.stickers.is_empty() {
                "none"
            } else {
                &joker.stickers
            }
        ))
}

pub(crate) fn consumable_tooltip(card: &ConsumableView) -> tooltip::Tip {
    tooltip::Tip::new(&card.name, &card.description)
        .info(info::references(card.kind))
        .badge(&card.category)
        .technical(format!("{} · index {}", card.kind, card.index))
        .technical(format!(
            "Price ${} · Sell ${} · can buy {} · can use {} · can sell {}",
            card.price, card.sell_price, card.can_buy, card.can_use, card.can_sell
        ))
}

pub(crate) fn inventory_cards(
    ui: &mut ChildSpawnerCommands,
    view: &RunView,
    atlases: &Atlases,
    playing_trace: bool,
    jokers: bool,
    selection: Option<(Lane, usize, u64)>,
) {
    let lane = if jokers {
        Lane::Jokers
    } else {
        Lane::Consumables
    };
    let (capacity, count, zone_width) = if jokers {
        (view.joker_slots, view.held_jokers.len(), 640.0)
    } else {
        (view.consumable_slots, view.consumables.len(), 320.0)
    };
    let width = (zone_width / capacity.max(count).max(1) as f32 - 4.0).clamp(48.0, 106.0);
    let revision = view.revision;
    let mut spawn = |index,
                     name: &str,
                     tip: tooltip::Tip,
                     sprite: Option<ImageNode>,
                     actions: &[(&str, Color, MvpAction)],
                     (edition, seed): (&str, usize),
                     joker: Option<JokerKey>| {
        let selected = !playing_trace && selection == Some((lane, index, revision));
        let mut card = ui.spawn(Node {
            width: px(width),
            height: px(width * 95.0 / 71.0),
            flex_shrink: 0.0,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        });
        if let Some(id) = joker {
            card.insert(ScoredJoker { id });
        } else if !jokers {
            card.insert(super::dissolve::HeldConsumable { index });
        }
        if selected {
            card.insert(UiTransform::from_xy(px(0), px(-10)));
        } else {
            card_hover(&mut card);
        }
        if !playing_trace {
            card_drag::attach(
                &mut card,
                lane,
                index,
                Some(Click::SelectInventory {
                    lane,
                    index,
                    revision,
                }),
                None,
                selected,
                index as i32,
            );
        }
        tooltip::attach(&mut card, tooltip::art(tip, sprite.as_ref()));
        card.with_children(|art| {
            if selected && !actions.is_empty() {
                let on_left = !jokers && index + 1 == count;
                art.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: if on_left { Val::Auto } else { percent(95) },
                        right: if on_left { percent(95) } else { Val::Auto },
                        top: percent(30),
                        padding: UiRect::all(px(3)),
                        row_gap: px(3),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    ZIndex(-1),
                    super::dissolve::InventoryActions,
                ))
                .observe(|mut event: On<PointerPress>| event.propagate(false))
                .with_children(|popup| {
                    for &(label, color, action) in actions {
                        menu_ui::tile_optional(
                            popup,
                            label,
                            color,
                            (0.98, 0.64),
                            0.23,
                            Some(Click::Play(action)),
                            false,
                        );
                    }
                });
            }
            if let Some(image) = sprite {
                // The edition draws over the sprite, so it is spawned after it.
                atlas_image(art, image.clone(), width, width * 95.0 / 71.0);
                super::edition::overlay(art, edition, &image, seed);
            } else {
                fallback_art(art, name, width);
            }
        });
    };
    if jokers {
        for (index, joker) in view.held_jokers.iter().enumerate() {
            let sell = format!("Sell ${}", joker.sell_price);
            let action = (
                sell.as_str(),
                menu_ui::PALE_GREEN,
                MvpAction::SellJoker { index, revision },
            );
            spawn(
                index,
                joker.name,
                joker_tooltip(joker),
                atlases.joker(joker.kind),
                if joker.can_sell {
                    std::slice::from_ref(&action)
                } else {
                    &[]
                },
                (&joker.edition, joker.id.0),
                Some(joker.id),
            );
        }
    } else {
        for (index, card) in view.consumables.iter().enumerate() {
            let sell = format!("Sell ${}", card.sell_price);
            let mut actions = Vec::new();
            if card.can_use {
                actions.push((
                    "Use",
                    menu_ui::BLUE,
                    MvpAction::UseConsumable {
                        index: card.index,
                        revision,
                    },
                ));
            }
            if card.can_sell {
                actions.push((
                    sell.as_str(),
                    menu_ui::PALE_GREEN,
                    MvpAction::SellConsumable {
                        index: card.index,
                        revision,
                    },
                ));
            }
            spawn(
                index,
                &card.name,
                consumable_tooltip(card),
                atlases.consumable(card.kind),
                &actions,
                ("", card.index),
                None,
            );
        }
    }
}

pub(crate) fn voucher_card(ui: &mut ChildSpawnerCommands, voucher: &Content, atlases: &Atlases) {
    info_box(
        ui,
        &voucher.name,
        tooltip::Tip::new(&voucher.name, &voucher.description)
            .badge("Voucher")
            .technical(format!("{} · ${}", voucher.id.0, voucher.cost))
            .technical(format!("Supported: {}", voucher.supported)),
        atlases.voucher(voucher.id.0),
        None,
        "Voucher",
    );
}

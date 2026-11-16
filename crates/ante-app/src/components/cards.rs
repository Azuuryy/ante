use super::{
    card_drag::{self, Lane},
    info,
    selection::{Layer, SelectKey, Selectable, SelectionButtons},
    tiles as menu_ui, tooltip,
    widgets::{atlas_image, card_hover, fallback_art},
};
use crate::{
    app::{presentation::ScoredJoker, session::Click},
    services::assets::catalog::Atlases,
};
use ante_core::contract::{Command, ConsumableView, Content, JokerKey, JokerView, RunView};
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

/// Stacking order of a selected card, above its neighbours and below the run overlays.
const SELECTED_Z: i32 = 100;

/// A button beside an inventory card: label rows, colour, minimum size in tiles, action.
struct CardAction {
    rows: Vec<Vec<(String, f32)>>,
    color: Color,
    size: (f32, f32),
    action: Option<Command>,
}

/// `G.UIDEF.use_and_sell_buttons`: green "SELL $n", inactive when the card cannot be sold.
fn sell_button(price: String, can_sell: bool, action: Command) -> CardAction {
    CardAction {
        rows: vec![
            vec![("SELL".to_owned(), 0.4)],
            vec![("$".to_owned(), 0.4), (price, 0.55)],
        ],
        color: menu_ui::GREEN,
        size: (1.25, 0.0),
        action: can_sell.then_some(action),
    }
}

/// Spawns the buttons of a selected inventory card.
fn card_actions(ui: &mut ChildSpawnerCommands, actions: &[CardAction]) {
    for button in actions {
        let rows: Vec<Vec<(&str, f32)>> = button
            .rows
            .iter()
            .map(|row| row.iter().map(|(t, s)| (t.as_str(), *s)).collect())
            .collect();
        let rows: Vec<&[(&str, f32)]> = rows.iter().map(Vec::as_slice).collect();
        menu_ui::card_button(
            ui,
            &rows,
            button.color,
            button.size,
            UiRect::all(menu_ui::tiles(0.1)),
            button.action.map(Click::Play),
        );
    }
}

pub(crate) fn inventory_cards(
    ui: &mut ChildSpawnerCommands,
    view: &RunView,
    atlases: &Atlases,
    playing_trace: bool,
    jokers: bool,
) {
    let lane = if jokers {
        Lane::Jokers
    } else {
        Lane::Consumables
    };
    let (capacity, count, area) = if jokers {
        (
            view.joker_slots,
            view.held_jokers.len(),
            menu_ui::JOKER_AREA,
        )
    } else {
        (
            view.consumable_slots,
            view.consumables.len(),
            menu_ui::CONSUMABLE_AREA,
        )
    };
    let card_size = menu_ui::card_px();
    let revision = view.revision;
    let mut spawn = |index,
                     name: &str,
                     tip: tooltip::Tip,
                     sprite: Option<ImageNode>,
                     actions: Vec<CardAction>,
                     (edition, seed): (&str, usize),
                     joker: Option<JokerKey>| {
        let mut card = ui.spawn(Node {
            position_type: PositionType::Absolute,
            left: px(menu_ui::spread_x(index, count, capacity, area, card_size.x)),
            top: px(0),
            width: px(card_size.x),
            height: px(card_size.y),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        });
        if let Some(id) = joker {
            card.insert(ScoredJoker { id });
        } else if !jokers {
            card.insert(super::dissolve::HeldConsumable { index });
        }
        card_hover(&mut card);
        if !playing_trace {
            // The selected card rises 10 px and comes to the front with its buttons until it
            // is deselected (`components::selection`).
            card.insert(Selectable {
                key: SelectKey::Inventory(lane, index, revision),
                layer: Layer::Global,
                rest_z: index as i32,
                chosen_z: SELECTED_Z,
                lift: 10.0,
            });
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
                false,
                index as i32,
            );
        }
        tooltip::attach(&mut card, tooltip::art(tip, sprite.as_ref()));
        card.with_children(|art| {
            if !playing_trace && !actions.is_empty() {
                // Aligned 'cr' with offset -0.4: the buttons start under the card's right
                // edge and are drawn behind it.
                art.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(card_size.x - menu_ui::tile_px(0.4)),
                        top: px(0),
                        bottom: px(0),
                        padding: UiRect::left(menu_ui::tiles(0.3)),
                        row_gap: menu_ui::tiles(0.05),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ZIndex(-1),
                    super::dissolve::InventoryActions,
                    SelectionButtons,
                ))
                .observe(|mut event: On<PointerPress>| event.propagate(false))
                .with_children(|popup| card_actions(popup, &actions));
            }
            if let Some(image) = sprite {
                // The edition draws over the sprite, so it is spawned after it.
                atlas_image(art, image.clone(), card_size.x, card_size.y);
                super::edition::overlay(art, edition, &image, seed);
            } else {
                fallback_art(art, name, card_size.x);
            }
        });
    };
    if jokers {
        for (index, joker) in view.held_jokers.iter().enumerate() {
            spawn(
                index,
                joker.name,
                joker_tooltip(joker),
                atlases.joker(joker.kind),
                vec![sell_button(
                    joker.sell_price.to_string(),
                    joker.can_sell,
                    Command::SellJoker { index, revision },
                )],
                (&joker.edition, joker.id.0),
                Some(joker.id),
            );
        }
    } else {
        for (index, card) in view.consumables.iter().enumerate() {
            // Consumables stack SELL above a red USE of at least 1.25 x 1 tiles.
            let actions = vec![
                sell_button(
                    card.sell_price.to_string(),
                    card.can_sell,
                    Command::SellConsumable {
                        index: card.index,
                        revision,
                    },
                ),
                CardAction {
                    rows: vec![vec![("USE".to_owned(), 0.5)]],
                    color: menu_ui::RED,
                    size: (1.25, 1.0),
                    action: card.can_use.then_some(Command::UseConsumable {
                        index: card.index,
                        revision,
                    }),
                },
            ];
            spawn(
                index,
                &card.name,
                consumable_tooltip(card),
                atlases.consumable(card.kind),
                actions,
                (super::edition::consumable_look(card.kind), card.index),
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

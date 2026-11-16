//! Views of a Joker or consumable, shared by the shop, the inventories and the open pack,
//! so each place shows the same popup (rarity, edition, stickers, set).

use super::*;
use balatro_rs::{consumable::Consumable, joker::Jokers};

/// A Joker's view, unpriced and not actionable; callers set price and actions.
pub(super) fn joker_view(joker: &Jokers, game: &Game) -> JokerView {
    JokerView {
        id: JokerKey(joker.instance_id()),
        kind: joker.id(),
        name: joker.name(),
        price: 0,
        enabled: false,
        description: joker_text(joker),
        state: format!("{} / {:?}", joker.state().counter, joker.state().selector),
        current: joker_current(joker, game),
        rarity: format!("{:?}", joker.rarity()),
        edition: format!("{:?}", joker.edition()),
        stickers: sticker_summary(joker.stickers()),
        can_sell: false,
        sell_price: game.joker_sell_value(joker),
    }
}

/// A consumable's view at `index`, unpriced and not actionable.
pub(super) fn consumable_view(card: Consumable, index: usize) -> ConsumableView {
    ConsumableView {
        index,
        kind: card.id(),
        name: card.name().to_owned(),
        category: consumable_set(card).to_owned(),
        description: consumable_description(card),
        price: 0,
        can_buy: false,
        can_buy_and_use: false,
        can_use: false,
        can_sell: false,
        sell_price: card.sell_value(),
    }
}

/// The set badge of a consumable, as the original writes it (`localize(set)`): the
/// engine's `type_label` is lower case.
pub(super) fn consumable_set(card: Consumable) -> &'static str {
    match card {
        Consumable::Tarot(_) => "Tarot",
        Consumable::Planet(_) => "Planet",
        Consumable::Spectral(_) => "Spectral",
    }
}

/// A card of the open pack, with the Joker or consumable view its popup is built from.
pub(super) fn pack_choice(content: &PackContent, game: &Game, enabled: bool) -> ShopItem {
    let (joker, consumable) = match content {
        PackContent::Joker(joker) => (Some(joker_view(joker, game)), None),
        PackContent::Tarot(card) => (None, Some(consumable_view(Consumable::Tarot(*card), 0))),
        PackContent::Planet(card) => (None, Some(consumable_view(Consumable::Planet(*card), 0))),
        PackContent::Spectral(card) => {
            (None, Some(consumable_view(Consumable::Spectral(*card), 0)))
        }
        PackContent::PlayingCard(_) => (None, None),
    };
    ShopItem {
        name: content.name(),
        description: pack_content_description(content),
        art: pack_content_art(content),
        price: 0,
        enabled,
        joker,
        consumable,
    }
}

//! Sounds of player actions, like the original button, highlight and shop callbacks.

use ante_core::contract::Command as MvpAction;
use bevy::prelude::*;

use crate::{
    app::session::{Click, Session},
    services::assets::sounds::{Sfx, jitter},
};

/// Runs before the session applies the clicks, so a toggle knows the card's current state.
pub(crate) fn click_sounds(
    mut clicks: MessageReader<Click>,
    session: Res<Session>,
    mut sounds: MessageWriter<Sfx>,
) {
    for click in clicks.read() {
        let sound = match click {
            Click::HoverDeck(_) | Click::SelectInventory { .. } | Click::ClearSelection => None,
            Click::Play(action) => Some(match action {
                // `Card:click` highlights with cardSlide1 and unhighlights with cardSlide2.
                MvpAction::ToggleCard(id) => {
                    let selected = session.confirmed.as_ref().is_some_and(|view| {
                        view.cards
                            .iter()
                            .any(|card| card.id == *id && card.selected)
                    });
                    Sfx::new(if selected { "cardSlide2" } else { "cardSlide1" }, 1.0, 0.3)
                }
                MvpAction::MoveCard { .. }
                | MvpAction::ReorderJoker { .. }
                | MvpAction::ReorderConsumable { .. } => Sfx::new("cardSlide1", 1.0, 0.3),
                // `buy_from_shop` and `sell_card` pay with coin2.
                MvpAction::BuyJoker(_)
                | MvpAction::BuyConsumable { .. }
                | MvpAction::BuyAndUseConsumable { .. }
                | MvpAction::BuyPlayingCard { .. }
                | MvpAction::BuyPack { .. }
                | MvpAction::RedeemVoucher { .. }
                | MvpAction::SellJoker { .. }
                | MvpAction::SellConsumable { .. }
                | MvpAction::Reroll => Sfx::new("coin2", 1.0, 1.0),
                MvpAction::UseConsumable { .. } => Sfx::new("tarot1", jitter(0.9, 0.2), 0.6),
                _ => Sfx::new("button", 1.0, 0.3),
            }),
            // `UIElement:click` plays the button sound.
            _ => Some(Sfx::new("button", 1.0, 0.3)),
        };
        if let Some(sound) = sound {
            sounds.write(sound);
        }
    }
}

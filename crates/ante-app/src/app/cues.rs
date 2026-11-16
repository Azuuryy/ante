//! Sounds of player actions, like the original button, highlight and shop callbacks.

use ante_core::contract::Command;
use bevy::prelude::*;

use crate::{
    app::session::{Click, Session},
    services::assets::sounds::{ScheduledSfx, Sfx, jitter},
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
                Command::ToggleCard(id) => {
                    let selected = session.confirmed.as_ref().is_some_and(|view| {
                        view.cards
                            .iter()
                            .any(|card| card.id == *id && card.selected)
                    });
                    Sfx::new(if selected { "cardSlide2" } else { "cardSlide1" }, 1.0, 0.3)
                }
                Command::MoveCard { .. }
                | Command::ReorderJoker { .. }
                | Command::ReorderConsumable { .. } => Sfx::new("cardSlide1", 1.0, 0.3),
                // `buy_from_shop` and `sell_card` pay with coin2.
                Command::BuyJoker(_)
                | Command::BuyConsumable { .. }
                | Command::BuyAndUseConsumable { .. }
                | Command::BuyPlayingCard { .. }
                | Command::BuyPack { .. }
                | Command::RedeemVoucher { .. }
                | Command::SellJoker { .. }
                | Command::SellConsumable { .. }
                | Command::Reroll => Sfx::new("coin2", 1.0, 1.0),
                Command::UseConsumable { .. } => Sfx::new("tarot1", jitter(0.9, 0.2), 0.6),
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

/// Delays of the `Card:open` sounds at game speed `speed`, in real seconds: the opening
/// waits 0.2 s (the price or `delay(0.2)`) then 0.4 s before `Card:explode` plays
/// `explosion_buildup1`, and `explosion_release1` follows after `0.9 × explode_time`, with
/// `explode_time = 1.3 × √GAMESPEED`. Event delays run on the game clock, which advances
/// `speed` times faster than real time.
pub(crate) fn pack_open_delays(speed: f32) -> (f32, f32) {
    let speed = speed.max(0.01);
    let buildup = (0.2 + 0.4) / speed;
    let release = buildup + 0.9 * 1.3 * speed.sqrt() / speed;
    (buildup, release)
}

/// Schedules the explosion sounds when a pack opens, bought in the shop or given by a tag.
pub(crate) fn pack_open_sounds(
    session: Res<Session>,
    time: Res<Time<Real>>,
    mut was_open: Local<bool>,
    mut scheduled: ResMut<ScheduledSfx>,
) {
    let open = session.confirmed.as_ref().is_some_and(|view| {
        view.phase == ante_core::contract::Phase::Pack && view.open_pack.is_some()
    });
    if open && !*was_open {
        let (buildup, release) = pack_open_delays(session.speed.factor());
        let now = time.elapsed_secs();
        scheduled.after(now, buildup, Sfx::new("explosion_buildup1", 1.0, 1.0));
        scheduled.after(now, release, Sfx::new("explosion_release1", 1.0, 1.0));
    }
    *was_open = open;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_sounds_follow_card_open_and_explode() {
        let (buildup, release) = pack_open_delays(1.0);
        assert!((buildup - 0.6).abs() < 1e-6);
        assert!((release - (0.6 + 1.17)).abs() < 1e-5);
        let (fast_buildup, fast_release) = pack_open_delays(4.0);
        assert!((fast_buildup - 0.15).abs() < 1e-6);
        assert!((fast_release - (0.15 + 0.585)).abs() < 1e-5);
    }
}

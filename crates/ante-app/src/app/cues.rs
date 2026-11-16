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
                | Command::SellConsumable { .. } => Sfx::new("coin2", 1.0, 1.0),
                // `reroll_shop` pays, then the new cards materialize.
                Command::Reroll => {
                    for sound in materialize_sounds() {
                        sounds.write(sound);
                    }
                    Sfx::new("coin2", 1.0, 1.0)
                }
                // `G.FUNCS.cash_out` plays coin7 with the button sound.
                Command::CashOut => {
                    sounds.write(Sfx::new("button", 1.0, 0.3));
                    Sfx::new("coin7", 1.0, 1.0)
                }
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
/// `explosion_buildup1`; `explosion_release1` follows after `0.9 × explode_time`, with
/// `explode_time = 1.3 × √GAMESPEED`, and the pack cards materialize `1.3 × √GAMESPEED`
/// after the explosion starts. Event delays run on the game clock, which advances `speed`
/// times faster than real time.
pub(crate) fn pack_open_delays(speed: f32) -> (f32, f32, f32) {
    let speed = speed.max(0.01);
    let buildup = (0.2 + 0.4) / speed;
    let release = buildup + 0.9 * 1.3 * speed.sqrt() / speed;
    let materialize = buildup + 1.3 * speed.sqrt() / speed;
    (buildup, release, materialize)
}

/// `Card:start_materialize`: `whoosh1` and a random `crumple`, once for a batch of cards.
pub(crate) fn materialize_sounds() -> [Sfx; 2] {
    let crumple = ["crumple1", "crumple2", "crumple3", "crumple4", "crumple5"]
        [(rand::random::<f32>() * 5.0) as usize % 5];
    [
        Sfx::new("whoosh1", jitter(0.6, 0.1), 0.3),
        Sfx::new(crumple, jitter(1.2, 0.2), 0.8),
    ]
}

/// The run's counters, to sound their changes like the original `ease_*` functions.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct Counters {
    run: u64,
    phase: ante_core::contract::Phase,
    pack: bool,
    money: usize,
    hands: usize,
    discards: usize,
    ante: usize,
    round: usize,
}

/// Sounds of a change between two views of the same run:
/// - a pack opening: `Card:open` explodes, then its cards materialize;
/// - the shop after a cash out: its cards materialize;
/// - money: `ease_dollars` plays `coin1` (a scored hand plays its own popups instead);
/// - hands or discards: `ease_hands_played` and `ease_discard` play `chips2`;
/// - ante: `ease_ante` plays `highlight2` and `generic1`;
/// - round: `ease_round` plays `timpani` and `generic1`.
///
/// Returns the sounds to play now and those to schedule (delay, sound).
pub(crate) fn counter_sounds(
    before: Counters,
    after: Counters,
    scoring: bool,
    speed: f32,
) -> (Vec<Sfx>, Vec<(f32, Sfx)>) {
    use ante_core::contract::Phase;
    let (mut now, mut later) = (Vec::new(), Vec::new());
    if after.pack && !before.pack {
        let (buildup, release, materialize) = pack_open_delays(speed);
        later.push((buildup, Sfx::new("explosion_buildup1", 1.0, 1.0)));
        later.push((release, Sfx::new("explosion_release1", 1.0, 1.0)));
        later.extend(materialize_sounds().map(|sound| (materialize, sound)));
    }
    if before.phase == Phase::Reward && after.phase == Phase::Shop {
        now.extend(materialize_sounds());
    }
    if after.money != before.money && !scoring {
        now.push(Sfx::new("coin1", 1.0, 1.0));
    }
    if after.hands != before.hands || after.discards != before.discards {
        now.push(Sfx::new("chips2", 1.0, 1.0));
    }
    if after.ante != before.ante {
        now.push(Sfx::new("highlight2", 0.685, 0.2));
        now.push(Sfx::new("generic1", 1.0, 1.0));
    }
    if after.round != before.round {
        now.push(Sfx::new("timpani", 0.8, 1.0));
        now.push(Sfx::new("generic1", 1.0, 1.0));
    }
    (now, later)
}

/// Plays [`counter_sounds`] as the confirmed view changes.
pub(crate) fn run_sounds(
    session: Res<Session>,
    time: Res<Time<Real>>,
    mut last: Local<Option<Counters>>,
    mut scheduled: ResMut<ScheduledSfx>,
    mut sounds: MessageWriter<Sfx>,
) {
    let Some(view) = session.confirmed.as_ref() else {
        *last = None;
        return;
    };
    let current = Counters {
        run: session.run_id,
        phase: view.phase,
        pack: view.phase == ante_core::contract::Phase::Pack && view.open_pack.is_some(),
        money: view.money,
        hands: view.hands,
        discards: view.discards,
        ante: view.ante,
        round: view.round,
    };
    let before = last.replace(current);
    let Some(before) = before.filter(|before| before.run == current.run) else {
        return;
    };
    if before == current {
        return;
    }
    let (now, later) = counter_sounds(
        before,
        current,
        session.playback.is_some(),
        session.speed.factor(),
    );
    for sound in now {
        sounds.write(sound);
    }
    let clock = time.elapsed_secs();
    for (delay, sound) in later {
        scheduled.after(clock, delay, sound);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ante_core::contract::Phase;

    fn counters(phase: Phase) -> Counters {
        Counters {
            run: 1,
            phase,
            pack: phase == Phase::Pack,
            money: 4,
            hands: 4,
            discards: 3,
            ante: 1,
            round: 1,
        }
    }

    fn names(sounds: &[Sfx]) -> Vec<&'static str> {
        sounds.iter().map(|sound| sound.name).collect()
    }

    #[test]
    fn pack_sounds_follow_card_open_and_explode() {
        let (buildup, release, materialize) = pack_open_delays(1.0);
        assert!((buildup - 0.6).abs() < 1e-6);
        assert!((release - (0.6 + 1.17)).abs() < 1e-5);
        assert!((materialize - (0.6 + 1.3)).abs() < 1e-5);
        let (fast_buildup, fast_release, _) = pack_open_delays(4.0);
        assert!((fast_buildup - 0.15).abs() < 1e-6);
        assert!((fast_release - (0.15 + 0.585)).abs() < 1e-5);
    }

    #[test]
    fn opening_a_pack_explodes_then_materializes_its_cards() {
        let (now, later) = counter_sounds(counters(Phase::Shop), counters(Phase::Pack), false, 1.0);
        assert!(now.is_empty());
        let later: Vec<_> = later.iter().map(|(_, sound)| sound.name).collect();
        assert_eq!(
            &later[..3],
            ["explosion_buildup1", "explosion_release1", "whoosh1"]
        );
    }

    #[test]
    fn counters_sound_like_the_ease_functions() {
        let before = counters(Phase::Reward);
        let after = Counters {
            phase: Phase::Shop,
            money: 10,
            ..before
        };
        let (now, _) = counter_sounds(before, after, false, 1.0);
        assert_eq!(names(&now)[0], "whoosh1", "the shop cards materialize");
        assert!(names(&now).contains(&"coin1"));

        let played = Counters {
            hands: 3,
            money: 12,
            ..counters(Phase::Playing)
        };
        let (now, _) = counter_sounds(counters(Phase::Playing), played, true, 1.0);
        assert_eq!(
            names(&now),
            ["chips2"],
            "a scored hand's money has its own popups"
        );

        let next_ante = Counters {
            ante: 2,
            round: 2,
            ..counters(Phase::BeforeBlind)
        };
        let (now, _) = counter_sounds(counters(Phase::BeforeBlind), next_ante, false, 1.0);
        assert_eq!(
            names(&now),
            ["highlight2", "generic1", "timpani", "generic1"]
        );
    }
}

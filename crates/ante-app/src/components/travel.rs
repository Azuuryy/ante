//! Playing cards travelling between the deck, the hand, the play area and the discard.
//!
//! The UI is rebuilt from the confirmed view, so a card entity does not survive a move.
//! Each card's last screen position is remembered by key; a rebuilt card starts from
//! there (or from the deck when it was just drawn) and eases to its new layout slot with
//! the original `Moveable:move_xy` velocity model. Cards that change area start one after
//! the other, 0.1 game seconds apart, like the original `draw_card` events.

use std::collections::HashMap;

use ante_core::contract::{CardKey, Command};
use bevy::{math::Affine2, prelude::*, ui::UiGlobalTransform, window::PrimaryWindow};

use crate::{
    app::{
        presentation::Screen,
        session::{Click, Session},
    },
    components::{
        room::room_frame,
        tiles::TILE,
        widgets::{HoverMotion, Pose},
    },
};

mod discard;
mod place;

pub(crate) use discard::*;
pub(crate) use place::*;

/// Original delay between two `draw_card` events, in game seconds.
pub(crate) const DRAW_DELAY: f32 = 0.1;
/// A remembered position older than this is from an earlier hand; the card comes from the deck.
const TRAIL_LIFETIME: f32 = 0.25;
/// The original discard pile sits off screen to the right, about 1.1 units above the play area.
const DISCARD_RISE: f32 = 1.1 * TILE;
/// Playing card width in UI pixels, so a discarded card leaves the screen entirely.
const CARD_WIDTH: f32 = super::tiles::CARD_WIDTH;

/// Deck pile on the board, where drawn cards come from.
#[derive(Component)]
pub(crate) struct DeckPile;

/// Travel state of a playing card, in logical pixels relative to its layout slot.
#[derive(Component)]
pub(crate) struct Travel {
    key: CardKey,
    played: bool,
    order: usize,
    ready: bool,
    /// A drawn card waiting for its turn stays hidden in the deck, as it is face down there.
    hidden: bool,
    delay: f32,
    velocity: Vec2,
    pub(crate) offset: Vec2,
    target: Vec2,
    discard: Vec2,
    start: Start,
    planned: bool,
    /// Layout centre (physical pixels) seen on the previous frame.
    layout: Option<Vec2>,
    /// Width and margins of a drawn card folded while it waits, so the hand only
    /// spreads over the cards that have arrived, like the original `CardArea`.
    folded: Option<(Val, UiRect)>,
    /// Real seconds since a drawn card started turning face up, while it turns.
    flip: Option<f32>,
    /// Plays `card1` when the card leaves, like each `draw_card` event.
    announce: bool,
}

/// The original `Moveable:move_wh` pinches the width at 8 card widths per real second,
/// so a flip closes in 0.125 s and reopens in 0.125 s.
const FLIP_HALF: f32 = 1.0 / 8.0;

/// Back of a drawn card, shown until the flip reaches its middle.
#[derive(Component)]
pub(crate) struct CardBack;

/// Where a rebuilt card starts from.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Start {
    Here,
    Seen(Vec2, Vec2, Pose),
    Deck,
}

impl Travel {
    pub(crate) fn new(key: CardKey, played: bool, order: usize) -> Self {
        Self {
            key,
            played,
            order,
            ready: false,
            hidden: false,
            delay: 0.0,
            velocity: Vec2::ZERO,
            offset: Vec2::ZERO,
            target: Vec2::ZERO,
            discard: Vec2::ZERO,
            start: Start::Here,
            planned: false,
            layout: None,
            folded: None,
            flip: None,
            announce: false,
        }
    }

    pub(crate) fn played(&self) -> bool {
        self.played
    }

    /// Horizontal scale of the card while it flips, 1 otherwise.
    pub(crate) fn flip_width(&self) -> f32 {
        match self.flip {
            Some(elapsed) => ((elapsed - FLIP_HALF).abs() / FLIP_HALF).min(1.0),
            None => 1.0,
        }
    }

    /// Whether the back still shows: a drawn card waiting or in the first half of its flip.
    fn back_shown(&self) -> bool {
        self.start == Start::Deck && self.flip.is_none_or(|elapsed| elapsed < FLIP_HALF)
    }

    /// Restores the width of a drawn card once it leaves the deck.
    pub(crate) fn unfold(&mut self, node: &mut Node) {
        if self.hidden {
            return;
        }
        if let Some((width, margin)) = self.folded.take() {
            node.width = width;
            node.margin = margin;
        }
    }

    /// Position of the card in its row, which is also its turn when it leaves.
    pub(crate) fn order(&self) -> usize {
        self.order
    }

    /// Whether the card should show; false while a drawn card waits in the deck.
    pub(crate) fn shown(&self) -> bool {
        !self.hidden
    }

    /// Whether a card sent to the discard pile has reached it.
    fn discarded(&self) -> bool {
        self.ready && self.target == self.discard && self.offset == self.target
    }

    /// Sends the card to the discard pile after `delay` real seconds.
    pub(crate) fn discard(&mut self, delay: f32) {
        if self.target != self.discard {
            self.target = self.discard;
            self.delay = delay;
            self.announce = true;
        }
    }

    /// One frame of the original velocity model: the velocity eases toward
    /// 35 × distance × dt with `exp(-50·dt)` and is capped at 70 units per second.
    /// Returns true on the frame the card leaves, for its `card1` sound.
    pub(crate) fn step(&mut self, dt: f32) -> bool {
        if !self.ready {
            return false;
        }
        if self.delay > 0.0 {
            self.delay -= dt;
            return false;
        }
        let departed = std::mem::take(&mut self.announce);
        self.hidden = false;
        // A drawn card turns face up as it enters the hand (`CardArea:emplace`).
        if self.start == Start::Deck {
            let elapsed = self.flip.map_or(0.0, |elapsed| elapsed + dt);
            self.flip = Some(elapsed).filter(|&elapsed| elapsed < 2.0 * FLIP_HALF);
            if self.flip.is_none() {
                self.start = Start::Here;
            }
        }
        let distance = self.target - self.offset;
        if distance.length() < 0.01 * TILE && self.velocity.length() < 0.01 * TILE {
            self.offset = self.target;
            self.velocity = Vec2::ZERO;
            return departed;
        }
        let keep = (-50.0 * dt).exp();
        self.velocity = keep * self.velocity + (1.0 - keep) * distance * 35.0 * dt;
        let cap = 70.0 * TILE * dt.min(1.0 / 20.0);
        if self.velocity.length() > cap {
            self.velocity = self.velocity.normalize() * cap;
        }
        self.offset += self.velocity;
        departed
    }
}

/// A card as last drawn: its travel position (physical pixels, without the fan, lift or
/// hover), its velocity and its eased look.
struct Seen {
    position: Vec2,
    velocity: Vec2,
    pose: Pose,
    played: bool,
    at: f32,
}

/// Last screen position (physical pixels) of every playing card shown recently.
#[derive(Resource, Default)]
pub(crate) struct CardTrail {
    seen: HashMap<CardKey, Seen>,
    /// Cards just sent to the discard; the original draws only after they have left.
    discarding: usize,
}

/// Decides, before layout, where each rebuilt card comes from and when it leaves.
/// Cards entering the play area or drawn from the deck leave one after the other,
/// 0.1 game seconds apart; a drawn card waiting for its turn is hidden and folded.
#[allow(clippy::type_complexity)]
pub(crate) fn prepare(
    mut commands: Commands,
    time: Res<Time<Real>>,
    session: Res<Session>,
    mut trail: ResMut<CardTrail>,
    deck: Query<Entity, With<DeckPile>>,
    (children, images): (Query<&Children>, Query<&ImageNode>),
    mut cards: Query<(Entity, &mut Travel, &mut Node, &mut Visibility)>,
) {
    let now = time.elapsed_secs();
    let deck = deck.iter().next();
    let has_deck = deck.is_some();
    // The deck pile shows the card back; drawn cards wear it until they flip.
    let back = deck.and_then(|deck| {
        children
            .iter_descendants(deck)
            .find_map(|child| images.get(child).ok().cloned())
    });
    let mut arrivals = Vec::new();
    for (entity, mut travel, ..) in &mut cards {
        if travel.planned {
            continue;
        }
        travel.planned = true;
        let seen = trail
            .seen
            .get(&travel.key)
            .filter(|seen| now - seen.at < TRAIL_LIFETIME);
        let (start, moved_area) = match seen {
            Some(seen) => (
                Start::Seen(seen.position, seen.velocity, seen.pose),
                seen.played != travel.played,
            ),
            None if has_deck => (Start::Deck, true),
            None => (Start::Here, false),
        };
        travel.start = start;
        if start == Start::Deck
            && let Some(back) = &back
        {
            commands.entity(entity).with_child((
                CardBack,
                back.clone(),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                ZIndex(20),
                Pickable::IGNORE,
            ));
        }
        if moved_area {
            arrivals.push((travel.played, travel.order, entity));
        }
    }
    if arrivals.is_empty() {
        return;
    }
    arrivals.sort_by_key(|&(played, order, _)| (!played, order));
    let speed = session.speed.factor();
    let queued = std::mem::take(&mut trail.discarding);
    for (index, (_, _, entity)) in arrivals.into_iter().enumerate() {
        if let Ok((_, mut travel, mut node, mut visibility)) = cards.get_mut(entity) {
            travel.delay = (queued + index) as f32 * DRAW_DELAY / speed;
            travel.announce = true;
            if travel.start == Start::Deck && travel.delay > 0.0 {
                travel.hidden = true;
                *visibility = Visibility::Hidden;
                travel.folded = Some((node.width, node.margin));
                node.width = px(0);
                node.margin = UiRect::ZERO;
            }
        }
    }
}

#[cfg(test)]
mod tests;

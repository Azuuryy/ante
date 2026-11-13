//! Playing cards travelling between the deck, the hand, the play area and the discard.
//!
//! The UI is rebuilt from the confirmed view, so a card entity does not survive a move.
//! Each card's last screen position is remembered by key; a rebuilt card starts from
//! there (or from the deck when it was just drawn) and eases to its new layout slot with
//! the original `Moveable:move_xy` velocity model. Cards that change area start one after
//! the other, 0.1 game seconds apart, like the original `draw_card` events.

use std::collections::HashMap;

use ante_core::contract::{CardKey, Command as MvpAction};
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

/// Right after layout, starts rebuilt cards from their previous position and keeps every
/// card where it was drawn when the layout around it changes, so it glides instead of
/// jumping. Then records where every card is drawn. Runs before clipping and visibility
/// propagation, so the first frame of a rebuilt card is already drawn at its old position.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(crate) fn place(
    time: Res<Time<Real>>,
    mut trail: ResMut<CardTrail>,
    windows: Query<&Window, With<PrimaryWindow>>,
    deck: Query<&UiGlobalTransform, (With<DeckPile>, Without<Travel>)>,
    mut cards: Query<(
        Entity,
        &mut Travel,
        &ComputedNode,
        &mut UiTransform,
        &mut UiGlobalTransform,
        Option<&mut HoverMotion>,
        Has<DiscardedCard>,
    )>,
    children: Query<&Children>,
    mut globals: Query<
        &mut UiGlobalTransform,
        (Without<Travel>, Without<DeckPile>, Without<Screen>),
    >,
    rooms: Query<
        (&UiGlobalTransform, &UiTransform, &ComputedNode),
        (With<Screen>, Without<Travel>),
    >,
) {
    let now = time.elapsed_secs();
    // Positions are kept in room space, so the room sway never reads as card movement.
    let room = room_frame(rooms.iter().next());
    let to_room = room.inverse();
    let deck = deck
        .iter()
        .next()
        .map(|deck| to_room.transform_point2(deck.affine().translation));
    let width = windows
        .single()
        .map(|window| window.resolution.physical_width() as f32)
        .unwrap_or(0.0);
    for (entity, mut travel, node, mut transform, mut global, motion, detached) in &mut cards {
        if !travel.planned || node.size().y == 0.0 {
            continue;
        }
        // A detached discarded card is no longer inside the room.
        let (frame, to_frame) = if detached {
            (Affine2::IDENTITY, Affine2::IDENTITY)
        } else {
            (room, to_room)
        };
        let scale = node.inverse_scale_factor();
        let drawn = Vec2::new(
            px_value(transform.translation.x),
            px_value(transform.translation.y),
        );
        let layout = to_frame.transform_point2(global.affine().translation) - drawn / scale;
        if travel.ready {
            // The layout moved under the card: keep it where it is and let it ease over.
            let moved = travel.layout.map_or(Vec2::ZERO, |last| layout - last);
            travel.layout = Some(layout);
            if moved.length() < 0.25 {
                continue;
            }
            let moved_logical = moved * scale;
            if travel.target == travel.discard {
                travel.target -= moved_logical;
            }
            travel.discard -= moved_logical;
            travel.offset -= moved_logical;
            let next = drawn - moved_logical;
            transform.translation = Val2::new(Val::Px(next.x), Val::Px(next.y));
            let shift = frame * Affine2::from_translation(-moved) * to_frame;
            apply(&mut global, shift);
            for child in children.iter_descendants(entity) {
                if let Ok(mut global) = globals.get_mut(child) {
                    apply(&mut global, shift);
                }
            }
            continue;
        }
        let (from, velocity, pose) = match travel.start {
            Start::Seen(position, velocity, pose) => (position, velocity, Some(pose)),
            Start::Deck => (deck.unwrap_or(layout), Vec2::ZERO, None),
            Start::Here => (layout, Vec2::ZERO, None),
        };
        travel.ready = true;
        travel.layout = Some(layout);
        travel.offset = (from - layout) * scale;
        travel.velocity = velocity;
        travel.discard = Vec2::new((width - layout.x) * scale + CARD_WIDTH, -DISCARD_RISE);
        let pose = match (pose, motion) {
            (Some(pose), Some(mut motion)) => {
                motion.bypass_change_detection().restore(pose);
                pose
            }
            _ => Pose {
                scale: 1.0,
                ..Pose::default()
            },
        };
        // Draw this first frame exactly as the card looked before the rebuild.
        let moved = travel.offset + Vec2::new(0.0, pose.drop - pose.lift);
        *transform = UiTransform {
            translation: Val2::new(Val::Px(moved.x), Val::Px(moved.y)),
            scale: Vec2::splat(pose.scale),
            rotation: Rot2::radians(pose.tilt),
        };
        let look = frame
            * Affine2::from_translation(layout + moved / scale)
            * Affine2::from_scale_angle_translation(Vec2::splat(pose.scale), pose.tilt, Vec2::ZERO)
            * Affine2::from_translation(-layout)
            * to_frame;
        apply(&mut global, look);
        for child in children.iter_descendants(entity) {
            if let Ok(mut global) = globals.get_mut(child) {
                apply(&mut global, look);
            }
        }
    }
    for (_, travel, node, _, global, motion, detached) in &cards {
        if travel.ready {
            let to_frame = if detached { Affine2::IDENTITY } else { to_room };
            // Where the card is drawn, without its fan drop and lift, which the pose restores.
            // A dragged card keeps the drag translation, so a drop starts where it was let go.
            let pose = motion.map(|motion| motion.pose()).unwrap_or_default();
            let look = Vec2::new(0.0, pose.drop - pose.lift) / node.inverse_scale_factor();
            trail.seen.insert(
                travel.key,
                Seen {
                    position: to_frame.transform_point2(global.affine().translation) - look,
                    velocity: travel.velocity,
                    pose,
                    played: travel.played,
                    at: now,
                },
            );
        }
    }
}

/// Hides a drawn card's back once its flip passes the middle.
pub(crate) fn turn_card_backs(
    cards: Query<&Travel>,
    mut backs: Query<(&ChildOf, &mut Visibility), With<CardBack>>,
) {
    for (parent, mut visibility) in &mut backs {
        let shown = cards
            .get(parent.parent())
            .is_ok_and(|travel| travel.back_shown());
        let next = if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != next {
            *visibility = next;
        }
    }
}

/// Card discarded with the Discard button. It is detached from the hand so it outlives the
/// rebuilt hand, and is removed once it reaches the off-screen discard pile.
#[derive(Component)]
pub(crate) struct DiscardedCard {
    age: f32,
}

/// A discarded card is removed after this many real seconds even if it is still moving.
const DISCARDED_LIFETIME: f32 = 3.0;

/// After a Discard, detaches the cards that left the hand and sends them to the discard
/// pile one by one, 0.1 game seconds apart, like `G.FUNCS.discard_cards_from_highlighted`.
/// Runs after the session applied the click and before the hand is rebuilt.
#[allow(clippy::type_complexity)]
pub(crate) fn detach_discards(
    mut commands: Commands,
    mut clicks: MessageReader<Click>,
    session: Res<Session>,
    mut trail: ResMut<CardTrail>,
    mut previous: Local<Vec<(CardKey, bool)>>,
    mut cards: Query<(
        Entity,
        &mut Travel,
        &mut Node,
        &ComputedNode,
        &UiGlobalTransform,
        &UiTransform,
    )>,
) {
    let discarded = clicks
        .read()
        .any(|click| matches!(click, Click::Play(MvpAction::Discard)));
    let hand: Vec<(CardKey, bool)> = session
        .confirmed
        .as_ref()
        .map(|view| {
            view.cards
                .iter()
                .map(|card| (card.id, card.selected))
                .collect()
        })
        .unwrap_or_default();
    if discarded {
        let leaving: Vec<CardKey> = previous
            .iter()
            .filter(|(key, selected)| *selected && !hand.iter().any(|(id, _)| id == key))
            .map(|(key, _)| *key)
            .collect();
        let mut ghosts: Vec<_> = cards
            .iter_mut()
            .filter(|(_, travel, ..)| travel.ready && leaving.contains(&travel.key))
            .collect();
        ghosts.sort_by_key(|(_, travel, ..)| travel.order);
        trail.discarding = ghosts.len();
        let speed = session.speed.factor();
        for (index, (entity, mut travel, mut node, computed, global, transform)) in
            ghosts.into_iter().enumerate()
        {
            pin_in_place(&mut node, computed, global, transform);
            travel.discard(index as f32 * DRAW_DELAY / speed);
            commands.entity(entity).remove::<ChildOf>().insert((
                DiscardedCard { age: 0.0 },
                GlobalZIndex(400),
                Pickable::IGNORE,
            ));
        }
    }
    if session.is_changed() {
        *previous = hand;
    }
}

/// Keeps a card where it is drawn once detached: its layout slot becomes an absolute
/// position at the root, and its own transform keeps applying on top.
pub(crate) fn pin_in_place(
    node: &mut Node,
    computed: &ComputedNode,
    global: &UiGlobalTransform,
    transform: &UiTransform,
) {
    let scale = computed.inverse_scale_factor();
    let size = computed.size() * scale;
    let shown = global.affine().translation * scale;
    let moved = Vec2::new(
        px_value(transform.translation.x),
        px_value(transform.translation.y),
    );
    let slot = shown - moved;
    node.position_type = PositionType::Absolute;
    node.left = px(slot.x - size.x / 2.0);
    node.top = px(slot.y - size.y / 2.0);
    node.margin = UiRect::ZERO;
}

/// Removes discarded cards once they reach the pile.
pub(crate) fn retire_discards(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut cards: Query<(Entity, &mut DiscardedCard, &Travel)>,
) {
    for (entity, mut card, travel) in &mut cards {
        card.age += time.delta_secs();
        if travel.discarded() || card.age > DISCARDED_LIFETIME {
            commands.entity(entity).despawn();
        }
    }
}

fn px_value(value: Val) -> f32 {
    match value {
        Val::Px(value) => value,
        _ => 0.0,
    }
}

fn apply(global: &mut UiGlobalTransform, transform: Affine2) {
    *global = (transform * global.affine()).into();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ready(offset: Vec2) -> Travel {
        let mut travel = Travel::new(CardKey(1), false, 0);
        travel.ready = true;
        travel.offset = offset;
        travel
    }

    #[test]
    fn a_card_settles_on_its_slot() {
        let mut travel = ready(Vec2::new(-400.0, 250.0));
        for _ in 0..120 {
            travel.step(1.0 / 60.0);
        }
        assert_eq!(travel.offset, Vec2::ZERO);
    }

    #[test]
    fn speed_is_capped_at_seventy_units_per_second() {
        let mut travel = ready(Vec2::new(-100.0 * TILE, 0.0));
        let dt = 1.0 / 60.0;
        for _ in 0..30 {
            travel.step(dt);
            assert!(travel.velocity.length() <= 70.0 * TILE * dt + 1e-3);
        }
    }

    #[test]
    fn a_delayed_card_waits_before_moving() {
        let mut travel = ready(Vec2::new(-200.0, 0.0));
        travel.delay = 0.1;
        travel.step(0.05);
        assert_eq!(travel.offset, Vec2::new(-200.0, 0.0));
        travel.step(0.06);
        travel.step(0.016);
        assert!(travel.offset.x > -200.0);
    }

    #[test]
    fn a_discarded_card_is_done_once_it_reaches_the_pile() {
        let mut travel = ready(Vec2::ZERO);
        travel.discard = Vec2::new(300.0, -DISCARD_RISE);
        assert!(!travel.discarded());
        travel.discard(0.0);
        for _ in 0..240 {
            travel.step(1.0 / 60.0);
        }
        assert!(travel.discarded());
    }

    #[test]
    fn a_drawn_card_flips_face_up_after_leaving_the_deck() {
        let mut travel = ready(Vec2::new(500.0, 0.0));
        travel.start = Start::Deck;
        travel.delay = 0.1;
        travel.step(0.05);
        assert!(travel.back_shown());
        assert_eq!(travel.flip_width(), 1.0);
        // The delay runs out, then the flip starts at full width and closes.
        travel.step(0.06);
        travel.step(0.06);
        assert_eq!(travel.flip_width(), 1.0);
        travel.step(0.06);
        assert!(travel.back_shown());
        assert!(travel.flip_width() < 1.0);
        travel.step(0.1);
        assert!(!travel.back_shown());
        for _ in 0..10 {
            travel.step(0.05);
        }
        assert_eq!(travel.flip_width(), 1.0);
        assert!(!travel.back_shown());
    }

    #[test]
    fn discard_targets_the_pile_once() {
        let mut travel = ready(Vec2::ZERO);
        travel.discard = Vec2::new(900.0, -DISCARD_RISE);
        travel.discard(0.3);
        travel.step(0.1);
        travel.discard(0.3);
        assert!((travel.delay - 0.2).abs() < 1e-6);
    }
}

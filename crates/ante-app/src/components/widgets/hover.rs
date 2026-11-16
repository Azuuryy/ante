use super::*;

/// Playing card size in UI pixels: 0.95 x (CARD_W, CARD_H) tiles.
pub(crate) const CARD_HEIGHT: f32 = menu_ui::CARD_HEIGHT;

pub(crate) const CARD_WIDTH: f32 = menu_ui::CARD_WIDTH;

/// The hand area is 6 cards wide and spreads its cards over at least the hand size.
pub(super) const HAND_AREA: f32 = 6.0 * menu_ui::CARD_W * menu_ui::TILE;

pub(super) const HAND_SLOTS: usize = 8;

/// The play area is 5.3 cards wide with 5 slots (`play_W`, `card_limit`).
pub(super) const PLAY_AREA: f32 = 5.3 * menu_ui::CARD_W * menu_ui::TILE;

pub(super) const PLAY_SLOTS: usize = 5;

/// Gap between neighbouring cards of a row, from `align_cards`: centres step
/// `(w − card_w) / (max(n, limit) − 1)`. The play area steps by the unscaled card width,
/// like the original, so drawn cards keep a small gap.
pub(crate) fn row_gap(count: usize, played: bool) -> f32 {
    if played {
        use crate::components::card_area::{AreaKind, CardArea};
        let card = menu_ui::CARD_W * menu_ui::TILE;
        let play = CardArea::new(AreaKind::Play, PLAY_AREA, card, PLAY_SLOTS);
        let count = count.max(2);
        play.slot(1, count, CARD_WIDTH).x - play.slot(0, count, CARD_WIDTH).x - CARD_WIDTH
    } else {
        let slots = count.max(HAND_SLOTS);
        (HAND_AREA - CARD_WIDTH) / (slots - 1) as f32 - CARD_WIDTH
    }
}

/// A highlighted card rises by 0.2 card heights, as in the original card areas.
pub(crate) const HIGHLIGHT_LIFT: f32 = 0.2 * CARD_HEIGHT;

/// The play area sits 3.6 room units above the hand area.
pub(crate) const PLAY_AREA_RISE: f32 = 3.6 * menu_ui::TILE;

/// Hover adds 0.05 to the card's base scale of 0.95.
pub(super) const HOVER_SCALE: f32 = 1.0 + 0.05 / 0.95;

pub(super) const SCALE_RATE: f32 = 60.0;

pub(super) const POSITION_RATE: f32 = 50.0;

pub(super) const ROTATION_RATE: f32 = 190.0;

pub(super) const JUICE_TIME: f32 = 0.4;

/// Eased hover scale, highlight lift and juice of a card, like the original `Moveable`.
#[derive(Component)]
pub(crate) struct HoverMotion {
    pub(super) hovered: bool,
    pub(super) scale: f32,
    pub(super) lift: f32,
    /// Target lift set by presentation, such as scoring cards in the play area.
    pub(crate) raise: f32,
    pub(super) juice: Option<Juice>,
    pub(super) fan: Fan,
    /// Eased fan tilt (radians) and drop (px): the original eases rotation with
    /// `exp(-190·dt)` and position with `exp(-50·dt)` toward the aligned slot.
    pub(super) tilt: f32,
    pub(super) drop: f32,
    /// Horizontal offset (px) toward the slot a drag moved the card to, eased like the
    /// original `Moveable` position so neighbours slide while a card is dragged over them.
    pub(super) shift: f32,
    pub(crate) shift_target: f32,
}

/// Eased look of a card that survives a rebuild of the hand.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Pose {
    pub(crate) scale: f32,
    pub(crate) lift: f32,
    pub(crate) tilt: f32,
    pub(crate) drop: f32,
}

/// Hand fan of the original `CardArea:align_cards`: a tilt and a drop that grow away
/// from the middle, plus a slow sway phased by the card's x position (room units).
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub(crate) struct Fan {
    pub(super) rotation: f32,
    pub(super) drop: f32,
    pub(super) phase: f32,
    /// The `consumeable` area's bob instead of the fan sway.
    pub(super) bob: bool,
}

impl Fan {
    /// Fan of card `position` (0-based) in a hand of `count` cards.
    pub(crate) fn hand(count: usize, position: usize) -> Self {
        use crate::components::card_area::{AreaKind, CardArea};
        let n = count.max(1) as f32;
        let k = position as f32 + 1.0;
        let width = 6.0 * HAND_CARD_W - HAND_CARD_W;
        let slot = CardArea::new(AreaKind::Hand, 6.0, 1.0, HAND_SLOTS).slot(position, count, 1.0);
        Self {
            rotation: slot.rotation,
            drop: slot.drop * menu_ui::TILE,
            phase: HAND_X + width * (k - 1.0) / (n - 1.0).max(1.0),
            bob: false,
        }
    }

    /// Resting tilt and drop of a card area slot (`card_area::Slot`, room units), swaying
    /// with the phase `x` like the original's `card.T.x`.
    pub(crate) fn area(slot: crate::components::card_area::Slot, x: f32) -> Self {
        Self {
            rotation: slot.rotation,
            drop: slot.drop * menu_ui::TILE,
            phase: x,
            bob: false,
        }
    }

    /// A card of a `consumeable` area, such as an open pack: it bobs by
    /// `0.05 × sin(2 × 1.666 × t + x)` units while it is not highlighted.
    pub(crate) fn bob(x: f32) -> Self {
        Self {
            phase: x,
            bob: true,
            ..Self::default()
        }
    }

    #[cfg(test)]
    pub(crate) fn rotation(self) -> f32 {
        self.rotation
    }

    #[cfg(test)]
    pub(crate) fn drop(self) -> f32 {
        self.drop
    }

    /// Rotation (radians) and downward offset (px) at real time `t`.
    pub(super) fn at(self, t: f32) -> (f32, f32) {
        if self == Self::default() {
            return (0.0, 0.0);
        }
        if self.bob {
            return (
                0.0,
                0.05 * menu_ui::TILE * (2.0 * 1.666 * t + self.phase).sin(),
            );
        }
        (
            self.rotation + 0.02 * (2.0 * t + self.phase).sin(),
            self.drop + 0.03 * menu_ui::TILE * (0.666 * t + self.phase).sin(),
        )
    }
}

/// Re-fans the hand over the cards present in it, like `CardArea:align_cards` with
/// `#self.cards`: a drawn card counts once it has left the deck, a discarded one no longer does.
pub(crate) fn fan_hand(
    mut cards: Query<
        (&Travel, &mut HoverMotion),
        Without<crate::components::travel::DiscardedCard>,
    >,
) {
    let mut hand: Vec<_> = cards
        .iter_mut()
        .filter(|(travel, _)| !travel.played() && travel.shown())
        .collect();
    hand.sort_by_key(|(travel, _)| travel.order());
    let count = hand.len();
    for (position, (_, mut motion)) in hand.into_iter().enumerate() {
        let fan = Fan::hand(count, position);
        if motion.fan != fan {
            motion.bypass_change_detection().fan = fan;
        }
    }
}

/// Original card width and hand x in room units, used for the fan phase.
pub(super) const HAND_CARD_W: f32 = 2.049;

pub(super) const HAND_X: f32 = 4.86;

impl Default for HoverMotion {
    fn default() -> Self {
        Self {
            hovered: false,
            scale: 1.0,
            lift: 0.0,
            raise: 0.0,
            juice: None,
            fan: Fan::default(),
            tilt: 0.0,
            drop: 0.0,
            shift: 0.0,
            shift_target: 0.0,
        }
    }
}

pub(super) struct Juice {
    pub(super) elapsed: f32,
    pub(super) scale: f32,
    pub(super) rotation: f32,
}

impl HoverMotion {
    pub(crate) fn hovered(&self) -> bool {
        self.hovered
    }

    /// A card resting in a fanned area row.
    pub(crate) fn fanned(fan: Fan) -> Self {
        Self { fan, ..default() }
    }

    /// Original `juice_up(scale, rotation)` on a card: both are scaled by 0.4.
    pub(crate) fn juice_up(&mut self, scale: f32, rotation: f32, clockwise: bool) {
        self.juice = Some(Juice {
            elapsed: 0.0,
            scale: scale * 0.4,
            rotation: rotation * 0.4 * if clockwise { 1.0 } else { -1.0 },
        });
    }

    pub(crate) fn pose(&self) -> Pose {
        Pose {
            scale: self.scale,
            lift: self.lift,
            tilt: self.tilt,
            drop: self.drop,
        }
    }

    /// Continues the look a rebuilt card had before, so selecting a card does not
    /// straighten and re-fan the whole hand.
    pub(crate) fn restore(&mut self, pose: Pose) {
        self.scale = pose.scale;
        self.lift = pose.lift;
        self.tilt = pose.tilt;
        self.drop = pose.drop;
    }

    pub(super) fn step(&mut self, dt: f32, now: f32) -> (f32, f32, f32) {
        let scale_k = 1.0 - (-dt * SCALE_RATE).exp();
        let position_k = 1.0 - (-dt * POSITION_RATE).exp();
        // A highlighted card stops bobbing, as `align_cards` only moves the others.
        let (tilt, drop) = if self.fan.bob && self.raise > 0.0 {
            (0.0, 0.0)
        } else {
            self.fan.at(now)
        };
        self.tilt += (tilt - self.tilt) * (1.0 - (-dt * ROTATION_RATE).exp());
        self.drop += (drop - self.drop) * position_k;
        let target = if self.hovered { HOVER_SCALE } else { 1.0 };
        self.scale += (target - self.scale) * scale_k;
        self.lift += (self.raise - self.lift) * position_k;
        self.shift += (self.shift_target - self.shift) * position_k;
        if (self.shift - self.shift_target).abs() < 0.05 {
            self.shift = self.shift_target;
        }
        if (self.scale - target).abs() < 1e-4 {
            self.scale = target;
        }
        if (self.lift - self.raise).abs() < 0.05 {
            self.lift = self.raise;
        }
        let (mut scale, mut rotation) = (self.scale, 0.0);
        if let Some(juice) = &mut self.juice {
            juice.elapsed += dt;
            let progress = (juice.elapsed / JUICE_TIME).min(1.0);
            let t = juice.elapsed;
            scale += juice.scale * (50.8 * t).sin() * (1.0 - progress).powi(3);
            rotation = juice.rotation * (40.8 * t).sin() * (1.0 - progress).powi(2);
            if progress >= 1.0 {
                self.juice = None;
            }
        }
        (scale, self.lift, rotation)
    }
}

pub(crate) fn card_hover(entity: &mut bevy::ecs::system::EntityCommands) {
    entity
        .insert((UiTransform::IDENTITY, HoverMotion::default()))
        .observe(
            |event: On<PointerOver>,
             drag: Option<Res<card_drag::CardDrag>>,
             mut motions: Query<&mut HoverMotion>,
             mut sounds: MessageWriter<Sfx>| {
                if drag.is_some_and(|drag| drag.holds(event.entity)) {
                    return;
                }
                if let Ok(mut motion) = motions.get_mut(event.entity) {
                    motion.hovered = true;
                    motion.juice_up(0.05, 0.03, rand::random());
                    // `Card:hover` plays paper1.
                    sounds.write(Sfx::new("paper1", jitter(0.9, 0.2), 0.35));
                }
            },
        )
        .observe(
            |event: On<PointerOut>, mut motions: Query<&mut HoverMotion>| {
                if let Ok(mut motion) = motions.get_mut(event.entity) {
                    motion.hovered = false;
                }
            },
        );
}

/// Eases every hovered, raised or travelling card; a card held by a drag keeps the drag transform.
#[allow(clippy::type_complexity)]
pub(crate) fn ease_hover(
    time: Res<Time<Real>>,
    drag: Option<Res<card_drag::CardDrag>>,
    mut cards: Query<(
        Entity,
        &mut HoverMotion,
        &mut UiTransform,
        Option<&mut Travel>,
        &mut Visibility,
        &mut Node,
    )>,
    mut sounds: MessageWriter<Sfx>,
) {
    let dt = time.delta_secs();
    let now = time.elapsed_secs();
    for (entity, mut motion, mut transform, travel, mut visibility, mut node) in &mut cards {
        if drag.as_ref().is_some_and(|drag| drag.holds(entity)) {
            continue;
        }
        let (offset, pinch, lean) = travel.map_or((Vec2::ZERO, 1.0, 0.0), |mut travel| {
            let travel = travel.bypass_change_detection();
            if travel.step(dt) {
                // `draw_card` plays card1 at 0.85 + 50% × 0.2 pitch and 0.6 volume.
                sounds.write(Sfx::new("card1", 0.95, 0.6));
            }
            if travel.shown() && *visibility == Visibility::Hidden {
                *visibility = Visibility::Inherited;
                travel.unfold(&mut node);
            }
            (travel.offset, travel.flip_width(), travel.lean())
        });
        let motion = motion.bypass_change_detection();
        let (scale, lift, rotation) = motion.step(dt, now);
        let (tilt, drop) = (motion.tilt, motion.drop);
        let next = UiTransform {
            translation: Val2::new(
                Val::Px(offset.x + motion.shift),
                Val::Px(offset.y + drop - lift),
            ),
            scale: Vec2::new(scale * pinch, scale),
            rotation: Rot2::radians(rotation + tilt + lean),
        };
        if *transform != next {
            *transform = next;
        }
    }
}

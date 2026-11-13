use super::{
    card_drag,
    theme::{BUTTON, DISABLED, INK},
    tiles as menu_ui, tooltip,
    travel::Travel,
};
use crate::{
    app::{playback::Speed, presentation::ScoredCard, session::Click},
    services::assets::sounds::{Sfx, jitter},
};
use ante_core::contract::CardView;
use bevy::{
    picking::prelude::{Pickable, PointerButton, PointerClick, PointerOut, PointerOver},
    prelude::*,
    text::FontSize,
};

pub(crate) fn click_target(entity: &mut bevy::ecs::system::EntityCommands, action: Click) {
    entity.observe(
        move |event: On<PointerClick>, mut clicks: MessageWriter<Click>| {
            if event.button == PointerButton::Primary {
                clicks.write(action);
            }
        },
    );
}

/// Playing card size in UI pixels: 0.95 x (CARD_W, CARD_H) tiles.
pub(crate) const CARD_HEIGHT: f32 = menu_ui::CARD_HEIGHT;
pub(crate) const CARD_WIDTH: f32 = menu_ui::CARD_WIDTH;
/// The hand area is 6 cards wide and spreads its cards over at least the hand size.
const HAND_AREA: f32 = 6.0 * menu_ui::CARD_W * menu_ui::TILE;
const HAND_SLOTS: usize = 8;
/// A highlighted card rises by 0.2 card heights, as in the original card areas.
pub(crate) const HIGHLIGHT_LIFT: f32 = 0.2 * CARD_HEIGHT;
/// The play area sits 3.6 room units above the hand area.
pub(crate) const PLAY_AREA_RISE: f32 = 3.6 * menu_ui::TILE;
/// Hover adds 0.05 to the card's base scale of 0.95.
const HOVER_SCALE: f32 = 1.0 + 0.05 / 0.95;
const SCALE_RATE: f32 = 60.0;
const POSITION_RATE: f32 = 50.0;
const ROTATION_RATE: f32 = 190.0;
const JUICE_TIME: f32 = 0.4;

/// Eased hover scale, highlight lift and juice of a card, like the original `Moveable`.
#[derive(Component)]
pub(crate) struct HoverMotion {
    hovered: bool,
    scale: f32,
    lift: f32,
    /// Target lift set by presentation, such as scoring cards in the play area.
    pub(crate) raise: f32,
    juice: Option<Juice>,
    fan: Fan,
    /// Eased fan tilt (radians) and drop (px): the original eases rotation with
    /// `exp(-190·dt)` and position with `exp(-50·dt)` toward the aligned slot.
    tilt: f32,
    drop: f32,
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
    rotation: f32,
    drop: f32,
    phase: f32,
}

impl Fan {
    /// Fan of card `position` (0-based) in a hand of `count` cards.
    pub(crate) fn hand(count: usize, position: usize) -> Self {
        let n = count.max(1) as f32;
        let k = position as f32 + 1.0;
        let width = 6.0 * HAND_CARD_W - HAND_CARD_W;
        Self {
            rotation: 0.2 * (-n / 2.0 - 0.5 + k) / n,
            drop: (0.5 * (-n / 2.0 + k - 0.5) / n).abs() * menu_ui::TILE,
            phase: HAND_X + width * (k - 1.0) / (n - 1.0).max(1.0),
        }
    }

    /// Rotation (radians) and downward offset (px) at real time `t`.
    fn at(self, t: f32) -> (f32, f32) {
        if self == Self::default() {
            return (0.0, 0.0);
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
    mut cards: Query<(&Travel, &mut HoverMotion), Without<super::travel::DiscardedCard>>,
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
const HAND_CARD_W: f32 = 2.049;
const HAND_X: f32 = 4.86;

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
        }
    }
}

struct Juice {
    elapsed: f32,
    scale: f32,
    rotation: f32,
}

impl HoverMotion {
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

    fn step(&mut self, dt: f32, now: f32) -> (f32, f32, f32) {
        let scale_k = 1.0 - (-dt * SCALE_RATE).exp();
        let position_k = 1.0 - (-dt * POSITION_RATE).exp();
        let (tilt, drop) = self.fan.at(now);
        self.tilt += (tilt - self.tilt) * (1.0 - (-dt * ROTATION_RATE).exp());
        self.drop += (drop - self.drop) * position_k;
        let target = if self.hovered { HOVER_SCALE } else { 1.0 };
        self.scale += (target - self.scale) * scale_k;
        self.lift += (self.raise - self.lift) * position_k;
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
        let (offset, pinch) = travel.map_or((Vec2::ZERO, 1.0), |mut travel| {
            let travel = travel.bypass_change_detection();
            if travel.step(dt) {
                // `draw_card` plays card1 at 0.85 + 50% × 0.2 pitch and 0.6 volume.
                sounds.write(Sfx::new("card1", 0.95, 0.6));
            }
            if travel.shown() && *visibility == Visibility::Hidden {
                *visibility = Visibility::Inherited;
                travel.unfold(&mut node);
            }
            (travel.offset, travel.flip_width())
        });
        let motion = motion.bypass_change_detection();
        let (scale, lift, rotation) = motion.step(dt, now);
        let (tilt, drop) = (motion.tilt, motion.drop);
        let next = UiTransform {
            translation: Val2::new(Val::Px(offset.x), Val::Px(offset.y + drop - lift)),
            scale: Vec2::new(scale * pinch, scale),
            rotation: Rot2::radians(rotation + tilt),
        };
        if *transform != next {
            *transform = next;
        }
    }
}

/// Drawn debuff cross and tint, hidden once the original debuff shader is loaded.
#[derive(Component)]
pub(crate) struct DebuffFallback;

/// Hides the drawn debuff look when the local debuff shader replaces it.
pub(crate) fn hide_debuff_fallback(
    shaders: Option<Res<crate::services::assets::shaders::LoveShaders>>,
    mut fallbacks: Query<&mut Node, Added<DebuffFallback>>,
) {
    if shaders.is_some_and(|shaders| shaders.get("debuff").is_some()) {
        for mut node in &mut fallbacks {
            node.display = Display::None;
        }
    }
}

/// A card shown in the play area while its hand is scored.
#[derive(Component)]
pub(crate) struct PlayedCard;

pub(crate) fn card_control(
    ui: &mut ChildSpawnerCommands,
    action: Option<Click>,
    card: &CardView,
    tip: tooltip::Tip,
    (count, position): (usize, usize),
    (reorderable, played): (bool, bool),
    content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    // `align_cards` steps (w − card_w)/(max(n, limit) − 1); the row stays centred.
    let slots = count.max(HAND_SLOTS);
    let spacing = (HAND_AREA - CARD_WIDTH) / (slots - 1) as f32 - CARD_WIDTH;
    let lifted = card.selected && !played;
    let mut entity = ui.spawn((
        Node {
            width: px(CARD_WIDTH),
            height: px(CARD_HEIGHT),
            flex_shrink: 0.0,
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(6)),
            margin: UiRect {
                left: px(if position > 0 { spacing } else { 0.0 }),
                top: px(if lifted { -HIGHLIGHT_LIFT } else { 0.0 }),
                ..default()
            },
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BorderColor::all(Color::NONE),
        UiTransform::IDENTITY,
        ScoredCard { id: card.id },
    ));
    tooltip::attach(
        &mut entity,
        tip.technical(format!("Card #{} · {}{}", card.id.0, card.rank, card.suit))
            .technical(format!(
                "Selected {} · enabled {} · debuffed {} · base chips {}",
                card.selected, card.enabled, card.debuffed, card.chips
            ))
            .technical(format!(
                "Enhancement {} · edition {} · seal {}",
                card.enhancement, card.edition, card.seal
            )),
    );
    card_hover(&mut entity);
    entity.insert(Travel::new(card.id, played, position));
    if played {
        entity.insert(PlayedCard);
    }
    if reorderable {
        card_drag::attach(
            &mut entity,
            card_drag::Lane::Hand,
            position,
            action,
            Some(card.id),
            card.selected,
            0,
        );
    }
    entity.with_children(content);
}

pub(crate) fn fallback_art(ui: &mut ChildSpawnerCommands, name: &str, width: f32) {
    ui.spawn((
        Node {
            width: px(width),
            height: px(width * 95.0 / 71.0),
            padding: UiRect::all(px(6)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(px(6)),
            ..default()
        },
        BackgroundColor(DISABLED),
        Pickable::IGNORE,
    ))
    .with_children(|art| label(art, name, 15.0, INK));
}

/// A centred row that never wraps: wrapped menu rows kept a one-line height and their
/// second line covered the next row.
pub(crate) fn row(ui: &mut ChildSpawnerCommands, content: impl FnOnce(&mut ChildSpawnerCommands)) {
    ui.spawn(Node {
        flex_shrink: 0.0,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        column_gap: px(10),
        ..default()
    })
    .with_children(content);
}

/// A row of cards that may wrap onto more lines (collection pages, vouchers, pack choices).
pub(crate) fn wrap_row(
    ui: &mut ChildSpawnerCommands,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    ui.spawn(Node {
        flex_wrap: FlexWrap::Wrap,
        flex_shrink: 0.0,
        justify_content: JustifyContent::Center,
        column_gap: px(10),
        row_gap: px(10),
        ..default()
    })
    .with_children(content);
}

pub(crate) fn label(
    ui: &mut ChildSpawnerCommands,
    text: impl Into<String>,
    font_size: f32,
    color: Color,
) {
    ui.spawn((
        Text::new(text),
        TextFont {
            font_size: FontSize::Px(font_size),
            ..default()
        },
        TextColor(color),
        Pickable::IGNORE,
    ));
}

/// Shadow of text marked `shadow = true` in the original: black at 30%, a little lower.
pub(crate) fn text_shadow() -> TextShadow {
    TextShadow {
        offset: Vec2::new(0.0, 0.04 * menu_ui::TILE),
        color: Color::srgba(0.0, 0.0, 0.0, 0.3),
    }
}

pub(crate) fn shadowed_label(
    ui: &mut ChildSpawnerCommands,
    text: impl Into<String>,
    font_size: f32,
    color: Color,
) {
    ui.spawn((
        Text::new(text),
        TextFont {
            font_size: FontSize::Px(font_size),
            ..default()
        },
        TextColor(color),
        text_shadow(),
        TextLayout::no_wrap(),
        Pickable::IGNORE,
    ));
}

pub(crate) fn button(
    ui: &mut ChildSpawnerCommands,
    text: impl Into<String>,
    action: Option<Click>,
) {
    menu_ui::menu_button(ui, &text.into(), BUTTON, action);
}

pub(crate) fn control(
    ui: &mut ChildSpawnerCommands,
    background: Color,
    action: Option<Click>,
    (width, height): (f32, f32),
    description: Option<tooltip::Tip>,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    let mut entity = ui.spawn((
        Node {
            min_width: px(width),
            min_height: px(height),
            padding: UiRect::axes(px(12), px(5)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            ..default()
        },
        BackgroundColor(background),
    ));
    if let Some(description) = description {
        tooltip::attach(&mut entity, description);
    }
    if let Some(action) = action {
        entity.observe(
            move |event: On<PointerClick>, mut clicks: MessageWriter<Click>| {
                if event.button == PointerButton::Primary {
                    clicks.write(action);
                }
            },
        );
    }
    entity.with_children(content);
}

pub(crate) fn speed_button(ui: &mut ChildSpawnerCommands, speed: Speed) {
    button(
        ui,
        format!("Speed {}", speed.label()),
        Some(Click::ChangeSpeed),
    );
}

pub(crate) fn atlas_image(
    ui: &mut ChildSpawnerCommands,
    image: ImageNode,
    width: f32,
    height: f32,
) {
    ui.spawn((
        image,
        Node {
            width: px(width),
            height: px(height),
            ..default()
        },
        Pickable::IGNORE,
    ));
}

/// A playing card as `Card:draw` layers it: the enhancement centre (or the plain base), the
/// rank and suit face unless it is a Stone card, the edition shader and the seal.
pub(crate) struct PlayingFace<'a> {
    pub(crate) rank: char,
    pub(crate) suit: &'a str,
    pub(crate) enhancement: &'a str,
    pub(crate) edition: &'a str,
    pub(crate) seal: &'a str,
    pub(crate) debuffed: bool,
    pub(crate) seed: usize,
}

/// Draws `card`; false when the local card art is missing, so the caller can fall back.
pub(crate) fn playing_card_face(
    ui: &mut ChildSpawnerCommands,
    atlases: &crate::services::assets::catalog::Atlases,
    card: &PlayingFace,
    width: f32,
) -> bool {
    let Some(face) = atlases.card(card.rank, card.suit) else {
        return false;
    };
    let stone = card.enhancement == "Stone";
    card_face_with_seal(
        ui,
        atlases.card_base_for(card.enhancement),
        (!stone).then_some(face),
        card.debuffed,
        width,
        Some((card.seal, atlases.seal(card.seal))),
        (card.edition, card.seed),
    );
    true
}

pub(crate) fn card_face_with_seal(
    ui: &mut ChildSpawnerCommands,
    base: Option<ImageNode>,
    face: Option<ImageNode>,
    debuffed: bool,
    width: f32,
    seal: Option<(&str, Option<ImageNode>)>,
    (edition, seed): (&str, usize),
) {
    ui.spawn(Node {
        width: px(width),
        height: px(width * 95.0 / 71.0),
        ..default()
    })
    .with_children(|layers| {
        let seal_sprite = seal.as_ref().and_then(|(_, sprite)| sprite.clone());
        // Like `Card:draw`, the edition shader covers the base and the face, under the seal.
        let sprites: Vec<ImageNode> = base.into_iter().chain(face).collect();
        for image in &sprites {
            spawn_layer(layers, image.clone());
        }
        for image in &sprites {
            super::edition::overlay(layers, edition, image, seed);
        }
        if let Some(image) = seal_sprite {
            spawn_layer(layers, image.clone());
            // `Card:draw` shines the Gold seal with the voucher shader.
            if seal.as_ref().is_some_and(|(name, _)| *name == "Gold") {
                super::edition::overlay(layers, "Voucher", &image, seed);
            }
        }
        fn spawn_layer(layers: &mut ChildSpawnerCommands, image: ImageNode) {
            layers.spawn((
                image,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                Pickable::IGNORE,
            ));
        }
        if let Some((name, None)) = seal {
            let color = match name {
                "Gold" => Some(Color::srgb(0.94, 0.65, 0.18)),
                "Red" => Some(Color::srgb(0.83, 0.16, 0.17)),
                "Blue" => Some(Color::srgb(0.13, 0.45, 0.88)),
                "Purple" => Some(Color::srgb(0.54, 0.22, 0.72)),
                _ => None,
            };
            if let Some(color) = color {
                layers.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(19),
                        top: percent(14),
                        width: px(width * 0.40),
                        height: px(width * 0.40),
                        border: UiRect::all(px(width * 0.025)),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(color),
                    BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.52)),
                    Pickable::IGNORE,
                ));
            }
        }
        if debuffed {
            // `Card:draw` covers a debuffed card's base and face with the debuff shader;
            // the drawn cross and tint stay as the fallback without local shaders.
            for image in &sprites {
                super::edition::overlay(layers, "Debuff", image, seed);
            }
            layers.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.88, 0.12, 0.10, 0.20)),
                Pickable::IGNORE,
                DebuffFallback,
            ));
            for rotation in [-0.78, 0.78] {
                layers.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(7),
                        top: percent(43),
                        width: percent(120),
                        height: percent(14),
                        ..default()
                    },
                    UiTransform {
                        rotation: Rot2::radians(rotation),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.88, 0.06, 0.08, 0.84)),
                    Pickable::IGNORE,
                    DebuffFallback,
                ));
            }
        }
    });
}

#[cfg(test)]
mod tests;

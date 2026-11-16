//! Score popups: `card_eval_status_text` and `attention_text` with their backdrop.

use super::*;
use ante_core::contract::MessageInk;

/// Sound of a popup, as in `card_eval_status_text`: pitch 0.8 + 0.2 × (0.9 to 1.1).
pub(super) fn popup_sound(popup: Popup) -> Sfx {
    let pitch = 0.8 + 0.2 * (0.9 + 0.2 * rand::random::<f32>());
    match popup {
        Popup::Again => Sfx::new("generic1", pitch, 1.0),
        Popup::Chips(_) => Sfx::new("chips1", pitch, 1.0),
        Popup::Mult(_) => Sfx::new("multhit1", pitch, 1.0),
        Popup::XMult(_) => Sfx::new("multhit2", pitch, 0.7),
        Popup::Money(_) => Sfx::new("coin3", pitch, 1.0),
        Popup::NotAllowed => Sfx::new("cardFan2", 1.0, 1.0),
        Popup::Message(..) => Sfx::new("generic1", pitch, 1.0),
    }
}

/// Popup label and color, as in `card_eval_status_text`: chips blue, mult and xmult red,
/// money yellow (red when lost), "Again!" in the filter orange.
pub(super) fn popup_text(popup: Popup) -> (String, Color) {
    match popup {
        Popup::Again => ("Again!".into(), POPUP_AGAIN),
        Popup::Chips(amount) => (format!("{amount:+}"), POPUP_CHIPS),
        Popup::Mult(amount) => (format!("{amount:+} Mult"), POPUP_MULT),
        Popup::XMult(hundredths) => (format!("X{} Mult", multiplier(hundredths)), POPUP_MULT),
        Popup::Money(amount) if amount < 0 => (format!("-${}", -amount), POPUP_MULT),
        Popup::Money(amount) => (format!("${amount}"), POPUP_MONEY),
        Popup::NotAllowed => ("Not Allowed!".into(), Color::WHITE),
        Popup::Message(message, ink) => (
            message.into(),
            match ink {
                MessageInk::Chips => POPUP_CHIPS,
                MessageInk::Mult => POPUP_MULT,
                MessageInk::Filter => POPUP_AGAIN,
            },
        ),
    }
}

/// A factor in hundredths written like the original: X2, X1.5, X1.25.
pub(super) fn multiplier(hundredths: usize) -> String {
    let text = format!("{}.{:02}", hundredths / 100, hundredths % 100);
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

pub(super) const POPUP_AGAIN: Color = Color::srgb(1.0, 0.604, 0.0);

pub(super) const POPUP_MONEY: Color = Color::srgb(0.953, 0.725, 0.357);

pub(super) const POPUP_CHIPS: Color = Color::srgb(0.0, 0.616, 1.0);

pub(super) const POPUP_MULT: Color = Color::srgb(0.996, 0.373, 0.333);

/// Original status text sits above played cards and below Jokers and consumables.
#[derive(Clone, Copy)]
pub(super) enum PopupSide {
    Above,
    Below,
}

/// Text scale of a popup in `card_eval_status_text`: chips and money at 1, the others at
/// 0.7.
pub(super) fn popup_scale(popup: Popup) -> f32 {
    match popup {
        Popup::Chips(_) | Popup::Money(_) => 1.0,
        Popup::NotAllowed => 0.9,
        _ => 0.7,
    }
}

/// `attention_text` with a `backdrop_colour`: white text with a shadow over a square of the
/// popup colour, one particle of `Particles{scale = 2.4, lifespan = 5, speed = 0}` that
/// turns slowly and grows, centred on the card's top (played cards) or bottom (Jokers).
pub(super) fn score_popup(popup: Popup, side: PopupSide) -> impl Bundle {
    let (label, color) = popup_text(popup);
    let (top, bottom) = match side {
        PopupSide::Above => (Val::Auto, percent(105)),
        PopupSide::Below => (percent(105), Val::Auto),
    };
    let size = menu_ui::text_px(popup_scale(popup));
    (
        ScorePopup,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top,
            bottom,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        GlobalZIndex(500),
        Pickable::IGNORE,
        children![
            (
                PopupBackdrop::new(rand::random::<f32>(), rand::random::<f32>()),
                Node {
                    position_type: PositionType::Absolute,
                    width: px(0),
                    height: px(0),
                    ..default()
                },
                BackgroundColor(color),
                UiTransform::IDENTITY,
                Pickable::IGNORE,
            ),
            (
                Text::new(label),
                TextFont {
                    font_size: bevy::text::FontSize::Px(size),
                    ..default()
                },
                TextColor(Color::WHITE),
                crate::components::widgets::text_shadow(),
                Pickable::IGNORE,
            )
        ],
    )
}

/// The square behind a popup: its age in game seconds, its facing and its slow spin.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub(crate) struct PopupBackdrop {
    age: f32,
    facing: f32,
    spin: f32,
}

impl PopupBackdrop {
    /// A square with a random facing and a spin of `0.2 × (0.5 − random)` radians a second.
    pub(super) fn new(facing: f32, spin: f32) -> Self {
        Self {
            age: 0.0,
            facing: facing * std::f32::consts::TAU,
            spin: 0.2 * (0.5 - spin),
        }
    }

    /// Side in room units at `age`: `min(2 × min(age / 5 × 2.4, 2.4 × (5 − age) / 5), 2.4)`.
    pub(super) fn side(age: f32) -> f32 {
        const SCALE: f32 = 2.4;
        const LIFESPAN: f32 = 5.0;
        (2.0 * (age / LIFESPAN * SCALE).min(SCALE * (LIFESPAN - age) / LIFESPAN)).clamp(0.0, SCALE)
    }
}

/// Grows and turns the popup squares on the game clock, like `Particles:move`.
pub(crate) fn animate_popup_backdrops(
    time: Res<Time<Real>>,
    session: Res<Session>,
    mut squares: Query<(&mut PopupBackdrop, &mut Node, &mut UiTransform)>,
) {
    let dt = time.delta_secs() * session.speed.factor();
    for (mut square, mut node, mut transform) in &mut squares {
        square.age += dt;
        square.facing += square.spin * dt;
        let side = px(PopupBackdrop::side(square.age) * menu_ui::TILE);
        if node.width != side {
            node.width = side;
            node.height = side;
        }
        transform.rotation = Rot2::radians(square.facing);
    }
}

/// `attention_text` at scale 0.9, centred one unit above the play area.
pub(super) fn play_area_popup() -> impl Bundle {
    (
        ScorePopup,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(-menu_ui::TILE),
            justify_content: JustifyContent::Center,
            ..default()
        },
        GlobalZIndex(500),
        Pickable::IGNORE,
        children![(
            Text::new("Not Allowed!"),
            TextFont {
                font_size: bevy::text::FontSize::Px(menu_ui::text_px(0.9)),
                ..default()
            },
            TextColor(Color::WHITE),
            crate::components::widgets::text_shadow(),
            Pickable::IGNORE,
        )],
    )
}

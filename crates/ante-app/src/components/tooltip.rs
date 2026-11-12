use super::{info::Info, theme::MUTED, widgets::label};
use ante_core::contract::CurrentValue;
use bevy::{
    ecs::system::EntityCommands,
    picking::prelude::{Pickable, PointerOut, PointerOver},
    prelude::*,
    window::PrimaryWindow,
};

const TOOLTIP_WIDTH: f32 = 225.0;
/// Side boxes (`info_queue`) hang left of the main popup, centred on it, like `align = 'cl'`.
const SIDE_WIDTH: f32 = 140.0;
const SIDE_GAP: f32 = 4.0;
/// `info_tip_from_rows` title box: the original grey lightened by 0.15.
const SIDE_PANEL: Color = Color::srgb(
    0x77 as f32 / 255.0,
    0x88 as f32 / 255.0,
    0x8B as f32 / 255.0,
);
const TEXT_DARK: Color = Color::srgb(
    0x4F as f32 / 255.0,
    0x63 as f32 / 255.0,
    0x67 as f32 / 255.0,
);
const TEXT_INACTIVE: Color = Color::srgba(
    0x88 as f32 / 255.0,
    0x88 as f32 / 255.0,
    0x88 as f32 / 255.0,
    0x99 as f32 / 255.0,
);
const PAPER: Color = Color::WHITE;
#[cfg(feature = "dev-tools")]
const DEBUG_PANEL: Color = Color::srgba(0.86, 0.88, 0.90, 0.96);
/// Room height and card height in room units, used for the original anchor rule.
const ROOM_HEIGHT_UNITS: f32 = 11.5;
const CARD_HEIGHT_UNITS: f32 = 2.4 * 47.0 / 41.0;
const GAP: f32 = 10.0;
const TOOLTIP_PANEL: Color = Color::srgb(0.231, 0.275, 0.296);
const TOOLTIP_OUTLINE: Color = Color::srgb(0.875, 0.890, 0.918);
const MULT: Color = Color::srgb(
    0xFE as f32 / 255.0,
    0x5F as f32 / 255.0,
    0x55 as f32 / 255.0,
);
const CHIPS: Color = Color::srgb(
    0x00 as f32 / 255.0,
    0x9D as f32 / 255.0,
    0xFF as f32 / 255.0,
);
const MONEY: Color = Color::srgb(
    0xF3 as f32 / 255.0,
    0xB9 as f32 / 255.0,
    0x58 as f32 / 255.0,
);
const CHANCE: Color = Color::srgb(
    0x4B as f32 / 255.0,
    0xC2 as f32 / 255.0,
    0x92 as f32 / 255.0,
);
const ATTENTION: Color = Color::srgb(
    0xFF as f32 / 255.0,
    0x9A as f32 / 255.0,
    0x00 as f32 / 255.0,
);
const RARITY_COMMON: Color = Color::srgb(
    0x00 as f32 / 255.0,
    0x9D as f32 / 255.0,
    0xFF as f32 / 255.0,
);
const RARITY_UNCOMMON: Color = Color::srgb(
    0x4B as f32 / 255.0,
    0xC2 as f32 / 255.0,
    0x92 as f32 / 255.0,
);
const RARITY_RARE: Color = Color::srgb(
    0xFE as f32 / 255.0,
    0x5F as f32 / 255.0,
    0x55 as f32 / 255.0,
);
const RARITY_LEGENDARY: Color = Color::srgb(
    0xB2 as f32 / 255.0,
    0x6C as f32 / 255.0,
    0xBB as f32 / 255.0,
);
const TAROT: Color = Color::srgb(
    0xA7 as f32 / 255.0,
    0x82 as f32 / 255.0,
    0xD1 as f32 / 255.0,
);
const PLANET: Color = Color::srgb(
    0x13 as f32 / 255.0,
    0xAF as f32 / 255.0,
    0xCE as f32 / 255.0,
);
const SPECTRAL: Color = Color::srgb(
    0x45 as f32 / 255.0,
    0x84 as f32 / 255.0,
    0xFA as f32 / 255.0,
);
const VOUCHER: Color = Color::srgb(
    0xFD as f32 / 255.0,
    0x68 as f32 / 255.0,
    0x2B as f32 / 255.0,
);
const BOOSTER: Color = Color::srgb(
    0x64 as f32 / 255.0,
    0x6E as f32 / 255.0,
    0xB7 as f32 / 255.0,
);
const EDITION: Color = Color::srgb(
    0x4C as f32 / 255.0,
    0xA8 as f32 / 255.0,
    0x93 as f32 / 255.0,
);
const DARK_EDITION: Color = Color::BLACK;
const LOCKED: Color = Color::srgb(
    0x4F as f32 / 255.0,
    0x56 as f32 / 255.0,
    0x58 as f32 / 255.0,
);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tip {
    pub title: String,
    effect: String,
    currently: Option<String>,
    badges: Vec<String>,
    details: Vec<String>,
    side: Vec<Info>,
    #[cfg(feature = "dev-tools")]
    technical: Vec<String>,
}

impl Tip {
    pub fn new(title: impl Into<String>, effect: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            effect: effect.into(),
            currently: None,
            badges: Vec::new(),
            details: Vec::new(),
            side: Vec::new(),
            #[cfg(feature = "dev-tools")]
            technical: Vec::new(),
        }
    }

    pub fn currently(mut self, value: CurrentValue) -> Self {
        self.currently = Some(currently_text(value));
        self
    }

    pub fn badge(mut self, text: impl Into<String>) -> Self {
        self.badges.push(text.into());
        self
    }

    /// Adds side boxes, skipping one already shown.
    pub(crate) fn info(mut self, boxes: impl IntoIterator<Item = Info>) -> Self {
        for info in boxes {
            if !self.side.contains(&info) {
                self.side.push(info);
            }
        }
        self
    }

    pub fn detail(mut self, text: impl Into<String>) -> Self {
        let text = text.into();
        if !text.is_empty() {
            self.details.push(text);
        }
        self
    }

    pub fn technical(self, text: impl Into<String>) -> Self {
        #[cfg(feature = "dev-tools")]
        {
            let mut tip = self;
            tip.technical.push(text.into());
            tip
        }
        #[cfg(not(feature = "dev-tools"))]
        {
            let _ = text;
            self
        }
    }
}

/// Formats the grey "(Currently …)" line; `effect_line` colors the value like the original.
fn currently_text(value: CurrentValue) -> String {
    match value {
        CurrentValue::Mult(amount) => format!("(Currently +{amount} Mult)"),
        CurrentValue::Chips(amount) => format!("(Currently +{amount} Chips)"),
        CurrentValue::XMult(hundredths) => {
            let whole = hundredths / 100;
            let fraction = hundredths % 100;
            let factor = match fraction {
                0 => whole.to_string(),
                f if f % 10 == 0 => format!("{whole}.{}", f / 10),
                f => format!("{whole}.{f:02}"),
            };
            format!("(Currently X{factor} Mult)")
        }
        CurrentValue::Money(amount) => format!("(Currently ${amount})"),
        CurrentValue::Count(amount) => format!("(Currently {amount})"),
    }
}

pub fn art(tip: Tip, image: Option<&ImageNode>) -> Tip {
    #[cfg(feature = "dev-tools")]
    {
        match image {
            Some(image) => match image.rect {
                Some(rect) => tip.technical(format!(
                    "Atlas pixels: {:.0}, {:.0} / {:.0} x {:.0}",
                    rect.min.x,
                    rect.min.y,
                    rect.width(),
                    rect.height()
                )),
                None => tip.technical("Full texture"),
            },
            None => tip.technical("Fallback: local texture or atlas entry unavailable"),
        }
    }
    #[cfg(not(feature = "dev-tools"))]
    {
        let _ = image;
        tip
    }
}

impl From<String> for Tip {
    fn from(text: String) -> Self {
        let (title, effect) = text.split_once('\n').unwrap_or((&text, ""));
        Self::new(title, effect)
    }
}

/// Where the popup sits relative to its source, following the original placement rule.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TipAnchor {
    /// Above the source, or below it when the source is near the top of the room.
    #[default]
    Auto,
    /// Left of the source, as for shop offers and deck views.
    Left,
}

#[derive(Resource, Default)]
pub struct TooltipState {
    active: Option<(Entity, Tip, TipAnchor)>,
}

impl TooltipState {
    pub fn dismiss(&mut self) {
        self.active = None;
    }
}

#[derive(Component)]
pub(crate) struct TooltipPanel {
    source: Entity,
    tip: Tip,
}

pub fn attach(entity: &mut EntityCommands, tip: impl Into<Tip>) {
    attach_anchored(entity, tip, TipAnchor::Auto);
}

pub fn attach_anchored(entity: &mut EntityCommands, tip: impl Into<Tip>, anchor: TipAnchor) {
    let tip = tip.into();
    entity
        .observe(
            move |event: On<PointerOver>, mut state: ResMut<TooltipState>| {
                state.active = Some((event.entity, tip.clone(), anchor));
            },
        )
        .observe(|event: On<PointerOut>, mut state: ResMut<TooltipState>| {
            if state
                .active
                .as_ref()
                .is_some_and(|(id, _, _)| *id == event.entity)
            {
                state.dismiss();
            }
        });
}

pub(crate) fn update(
    mut commands: Commands,
    state: Res<TooltipState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut panels: Query<(Entity, &TooltipPanel, &ComputedNode, &mut Node)>,
    anchors: Query<(&ComputedNode, &UiGlobalTransform), Without<TooltipPanel>>,
    drag: Option<Res<super::card_drag::CardDrag>>,
) {
    let Ok(window) = windows.single() else { return };
    let active = state.active.as_ref().filter(|(source, _, _)| {
        !drag.as_ref().is_some_and(|drag| drag.moving())
            && anchors.contains(*source)
            && window.cursor_position().is_some()
    });
    let current = panels.single_mut().ok();
    let Some((source, tip, anchor)) = active else {
        if let Some((entity, _, _, _)) = current {
            commands.entity(entity).despawn();
        }
        return;
    };
    let Ok((node, transform)) = anchors.get(*source) else {
        return;
    };
    let dpi = window.scale_factor();
    let half = node.size() / dpi / 2.0;
    let center = transform.translation / dpi;
    let width = window.width().min(TOOLTIP_WIDTH);
    let side = if tip.side.is_empty() {
        0.0
    } else {
        SIDE_WIDTH + SIDE_GAP
    };
    // The main popup follows the original anchor rule; side boxes extend to its left.
    let place = |height: f32| {
        let pos = position(
            center,
            half,
            Vec2::new(width, height),
            window.size(),
            *anchor,
        );
        Vec2::new((pos.x - side).max(0.0), pos.y)
    };
    if let Some((entity, existing, measured, mut layout)) = current {
        if existing.source == *source && existing.tip == *tip {
            let pos = place(measured.size().y / dpi);
            layout.left = px(pos.x);
            layout.top = px(pos.y);
            return;
        }
        commands.entity(entity).despawn();
    }
    let pos = place(150.0);
    commands
        .spawn((
            TooltipPanel {
                source: *source,
                tip: tip.clone(),
            },
            Node {
                position_type: PositionType::Absolute,
                left: px(pos.x),
                top: px(pos.y),
                width: px(width + side),
                max_height: px(window.height()),
                row_gap: px(4),
                align_items: AlignItems::FlexEnd,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            GlobalZIndex(1000),
            Pickable::IGNORE,
        ))
        .with_children(|stack| {
            // Side boxes are centred on the main popup only, like `align = 'cl'`.
            stack
                .spawn((
                    Node {
                        column_gap: px(SIDE_GAP),
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|row| {
                    if !tip.side.is_empty() {
                        row.spawn((
                            Node {
                                width: px(SIDE_WIDTH),
                                flex_shrink: 0.0,
                                row_gap: px(4),
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                            Pickable::IGNORE,
                        ))
                        .with_children(|boxes| {
                            for info in &tip.side {
                                side_box(boxes, info);
                            }
                        });
                    }
                    row.spawn((
                        Node {
                            width: px(width),
                            flex_shrink: 0.0,
                            padding: UiRect::all(px(6)),
                            border: UiRect::all(px(2)),
                            border_radius: BorderRadius::all(px(9)),
                            row_gap: px(4),
                            flex_direction: FlexDirection::Column,
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        BackgroundColor(TOOLTIP_PANEL),
                        BorderColor::all(TOOLTIP_OUTLINE),
                        Pickable::IGNORE,
                    ))
                    .with_children(|panel| main_panel(panel, tip));
                });
            #[cfg(feature = "dev-tools")]
            if !tip.technical.is_empty() {
                stack
                    .spawn((
                        Node {
                            width: px(width),
                            padding: UiRect::all(px(6)),
                            border_radius: BorderRadius::all(px(6)),
                            row_gap: px(2),
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        BackgroundColor(DEBUG_PANEL),
                        Pickable::IGNORE,
                    ))
                    .with_children(|debug| {
                        for line in &tip.technical {
                            label(debug, line, 14.0, TEXT_DARK);
                        }
                    });
            }
        });
}

/// One `info_tip_from_rows` box: a light frame, a grey title band and a white rule box.
fn side_box(ui: &mut ChildSpawnerCommands, info: &Info) {
    ui.spawn((
        Node {
            width: percent(100),
            padding: UiRect::all(px(3)),
            border_radius: BorderRadius::all(px(7)),
            ..default()
        },
        BackgroundColor(TOOLTIP_OUTLINE),
        Pickable::IGNORE,
    ))
    .with_children(|frame| {
        frame
            .spawn((
                Node {
                    width: percent(100),
                    padding: UiRect::all(px(3)),
                    border_radius: BorderRadius::all(px(6)),
                    row_gap: px(3),
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(SIDE_PANEL),
                Pickable::IGNORE,
            ))
            .with_children(|panel| {
                label(panel, info.title, 16.0, Color::WHITE);
                panel
                    .spawn((
                        Node {
                            width: percent(100),
                            padding: UiRect::all(px(5)),
                            border_radius: BorderRadius::all(px(5)),
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        BackgroundColor(PAPER),
                        Pickable::IGNORE,
                    ))
                    .with_children(|rule| effect_line(rule, info.body, TEXT_DARK));
            });
    });
}

fn main_panel(panel: &mut ChildSpawnerCommands, tip: &Tip) {
    label(panel, &tip.title, 23.0, Color::WHITE);
    if !tip.effect.is_empty() || tip.currently.is_some() {
        panel
            .spawn((
                Node {
                    width: percent(100),
                    padding: UiRect::all(px(7)),
                    border_radius: BorderRadius::all(px(6)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(3),
                    ..default()
                },
                BackgroundColor(PAPER),
                Pickable::IGNORE,
            ))
            .with_children(|effect| {
                for line in tip.effect.lines() {
                    effect_line(effect, line, TEXT_DARK);
                }
                if let Some(currently) = &tip.currently {
                    effect_line(effect, currently, TEXT_INACTIVE);
                }
            });
    }
    if !tip.badges.is_empty() {
        panel
            .spawn((
                Node {
                    width: percent(100),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(3),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|badges| {
                for badge in &tip.badges {
                    let rarity = is_rarity_badge(badge);
                    badges
                        .spawn((
                            Node {
                                min_width: if rarity { px(148) } else { px(0) },
                                padding: UiRect::axes(
                                    px(if rarity { 14.0 } else { 7.0 }),
                                    px(if rarity { 4.0 } else { 2.0 }),
                                ),
                                justify_content: JustifyContent::Center,
                                border_radius: BorderRadius::all(px(5)),
                                ..default()
                            },
                            BackgroundColor(badge_color(badge)),
                            Pickable::IGNORE,
                        ))
                        .with_children(|pill| {
                            label(pill, badge, if rarity { 19.0 } else { 16.0 }, Color::WHITE)
                        });
                }
            });
    }
    for detail in &tip.details {
        label(panel, detail, 15.0, MUTED);
    }
}

fn is_rarity_badge(badge: &str) -> bool {
    matches!(badge, "Common" | "Uncommon" | "Rare" | "Legendary")
}

fn badge_color(badge: &str) -> Color {
    match badge {
        "Common" => RARITY_COMMON,
        "Uncommon" => RARITY_UNCOMMON,
        "Rare" => RARITY_RARE,
        "Legendary" => RARITY_LEGENDARY,
        "Tarot" => TAROT,
        "Planet" => PLANET,
        "Spectral" => SPECTRAL,
        "Voucher" => VOUCHER,
        "Booster" => BOOSTER,
        "Edition" => EDITION,
        "Foil" | "Holographic" | "Polychrome" | "Negative" => DARK_EDITION,
        "Locked" => LOCKED,
        _ => crate::components::tiles::BLUE,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ink {
    Base,
    Mult,
    Chips,
    Money,
    Chance,
    Attention,
}

fn effect_line(ui: &mut ChildSpawnerCommands, line: &str, base: Color) {
    let words: Vec<_> = line.split_whitespace().collect();
    let normalized: Vec<_> = words
        .iter()
        .map(|word| {
            word.trim_matches(|c: char| !c.is_alphanumeric() && c != '$')
                .to_ascii_lowercase()
        })
        .collect();
    ui.spawn((
        Node {
            flex_wrap: FlexWrap::Wrap,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(4),
            row_gap: px(2),
            ..default()
        },
        Pickable::IGNORE,
    ))
    .with_children(|row| {
        for (index, word) in words.into_iter().enumerate() {
            let word_key = &normalized[index];
            let multiplier = is_multiplier(&normalized, index, word);
            let color = match word_key.as_str() {
                "heart" | "hearts" => Color::srgb(0.92, 0.06, 0.19),
                "diamond" | "diamonds" => Color::srgb(0.74, 0.32, 0.04),
                "club" | "clubs" => Color::srgb(0.00, 0.40, 0.27),
                "spade" | "spades" => Color::srgb(0.35, 0.20, 0.54),
                "planet" | "planets" => Color::srgb(0.00, 0.37, 0.55),
                _ => ink_color(word_ink(word, word_key, &normalized, index), base),
            };
            if multiplier {
                let explicit_times =
                    word_key.starts_with('x') || word_key.ends_with('x') || word.contains('×');
                let display = if explicit_times {
                    word.to_owned()
                } else {
                    format!("×{word}")
                };
                row.spawn((
                    Node {
                        padding: UiRect::axes(px(3), px(0)),
                        border_radius: BorderRadius::all(px(3)),
                        ..default()
                    },
                    BackgroundColor(color),
                    Pickable::IGNORE,
                ))
                .with_children(|pill| label(pill, display, 17.0, Color::WHITE));
            } else {
                label(row, word, 17.0, color);
            }
        }
    });
}

fn word_ink(word: &str, key: &str, normalized: &[String], index: usize) -> Ink {
    if word.contains('$') {
        return Ink::Money;
    }
    if key.contains("chance") || is_chance_token(normalized, index) {
        return Ink::Chance;
    }
    let next = normalized.get(index + 1).map_or("", String::as_str);
    let previous = index.checked_sub(1).map_or("", |i| normalized[i].as_str());
    let previous_unit = index.checked_sub(2).and_then(|i| unit_ink(&normalized[i]));
    if numeric_word(word) {
        if let Some(unit) = unit_ink(next)
            .or_else(|| unit_ink(previous))
            .or(previous_unit)
        {
            return unit;
        }
        if is_multiplier(normalized, index, word) {
            return Ink::Mult;
        }
        return Ink::Attention;
    }
    if is_number_word(key) || is_rank_word(key) {
        return Ink::Attention;
    }
    Ink::Base
}

fn is_multiplier(normalized: &[String], index: usize, word: &str) -> bool {
    let key = &normalized[index];
    let next = normalized.get(index + 1).map_or("", String::as_str);
    let previous = index.checked_sub(1).map_or("", |i| normalized[i].as_str());
    let previous_unit = index.checked_sub(2).and_then(|i| unit_ink(&normalized[i]));
    let explicit_times = key.starts_with('x') || key.ends_with('x') || word.contains('×');
    numeric_word(word)
        && (explicit_times || next == "multiplier" || (previous == "by" && previous_unit.is_some()))
}

fn ink_color(ink: Ink, base: Color) -> Color {
    match ink {
        Ink::Base => base,
        Ink::Mult => MULT,
        Ink::Chips => CHIPS,
        Ink::Money => MONEY,
        Ink::Chance => CHANCE,
        Ink::Attention => ATTENTION,
    }
}

fn unit_ink(word: &str) -> Option<Ink> {
    match word {
        "mult" => Some(Ink::Mult),
        "chip" | "chips" => Some(Ink::Chips),
        _ => None,
    }
}

fn numeric_word(word: &str) -> bool {
    let value = word
        .trim_matches(|c: char| !c.is_alphanumeric())
        .trim_start_matches(['x', 'X'])
        .trim_end_matches(['x', 'X']);
    !value.is_empty()
        && value.bytes().any(|c| c.is_ascii_digit())
        && value.bytes().all(|c| c.is_ascii_digit() || c == b'.')
}

fn is_number_word(word: &str) -> bool {
    matches!(
        word,
        "one"
            | "two"
            | "three"
            | "four"
            | "five"
            | "six"
            | "seven"
            | "eight"
            | "nine"
            | "ten"
            | "eleven"
            | "twelve"
            | "thirteen"
            | "fourteen"
            | "fifteen"
            | "twenty"
            | "thirty"
            | "forty"
            | "fifty"
            | "hundred"
            | "thousand"
    )
}

fn is_rank_word(word: &str) -> bool {
    matches!(
        word,
        "ace" | "aces" | "jack" | "jacks" | "queen" | "queens" | "king" | "kings"
    )
}

fn is_chance_token(normalized: &[String], index: usize) -> bool {
    let key = &normalized[index];
    if key.contains("-in-") {
        let mut parts = key.split("-in-");
        if let (Some(left), Some(right), None) = (parts.next(), parts.next(), parts.next())
            && (is_number_word(left) || numeric_word(left))
            && (is_number_word(right) || numeric_word(right))
        {
            return true;
        }
    }
    let previous = index.checked_sub(1).map_or("", |i| normalized[i].as_str());
    let next = normalized.get(index + 1).map_or("", String::as_str);
    if key == "in" {
        return is_number_word(next) || numeric_word(next);
    }
    if is_number_word(key) || numeric_word(key) {
        if next == "in" {
            return normalized
                .get(index + 2)
                .is_some_and(|value| is_number_word(value) || numeric_word(value));
        }
        if previous == "in" && index >= 2 {
            let before = &normalized[index - 2];
            return is_number_word(before) || numeric_word(before);
        }
    }
    false
}

/// Original rule: above the card, below it when its top is within 0.8 card heights of
/// the room top, left of it for shop offers; flipped then clamped to stay on screen.
fn position(center: Vec2, half: Vec2, size: Vec2, window: Vec2, anchor: TipAnchor) -> Vec2 {
    let unit = window.y / ROOM_HEIGHT_UNITS;
    let pos = match anchor {
        TipAnchor::Left => {
            let left = center.x - half.x - GAP - size.x;
            let x = if left >= 0.0 {
                left
            } else {
                center.x + half.x + GAP
            };
            Vec2::new(x, center.y - size.y / 2.0)
        }
        TipAnchor::Auto => {
            let above = center.y - half.y - GAP - size.y;
            let below = center.y + half.y + GAP;
            let near_top = center.y - half.y < 0.8 * CARD_HEIGHT_UNITS * unit;
            let y = if (near_top || above < 0.0) && below + size.y <= window.y {
                below
            } else {
                above
            };
            Vec2::new(center.x - size.x / 2.0, y)
        }
    };
    pos.clamp(Vec2::ZERO, (window - size).max(Vec2::ZERO))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn side_boxes_keep_their_order_without_repeats() {
        let tip = Tip::new("Wheel", "")
            .info(crate::components::info::references("c_wheel_of_fortune"))
            .info(crate::components::info::edition("Foil", false));
        let titles: Vec<_> = tip.side.iter().map(|info| info.title).collect();
        assert_eq!(titles, ["Foil", "Holographic", "Polychrome"]);
    }

    fn inks(line: &str) -> Vec<Ink> {
        let words: Vec<_> = line.split_whitespace().collect();
        let normalized: Vec<_> = words
            .iter()
            .map(|word| {
                word.trim_matches(|c: char| !c.is_alphanumeric() && c != '$')
                    .to_ascii_lowercase()
            })
            .collect();
        words
            .iter()
            .enumerate()
            .map(|(index, word)| word_ink(word, &normalized[index], &normalized, index))
            .collect()
    }

    #[test]
    fn chance_phrases_are_green() {
        assert_eq!(
            inks("1 in 5 chance"),
            vec![Ink::Chance, Ink::Chance, Ink::Chance, Ink::Chance]
        );
        assert_eq!(inks("one-in-two"), vec![Ink::Chance]);
    }

    #[test]
    fn card_counts_are_not_mistaken_for_chances() {
        let ink = inks("9 in your full deck");
        assert_eq!(ink[0], Ink::Attention);
        assert!(!ink.contains(&Ink::Chance));
    }

    #[test]
    fn money_and_scoring_values_use_balatro_colors() {
        assert_eq!(inks("Earn $4")[1], Ink::Money);
        assert_eq!(inks("+50 Chips")[0], Ink::Chips);
        assert_eq!(inks("+4 Mult")[0], Ink::Mult);
        assert_eq!(inks("×2")[0], Ink::Mult);
    }

    #[test]
    fn ranks_and_counts_use_attention_orange() {
        assert_eq!(inks("Each Queen")[1], Ink::Attention);
        assert_eq!(inks("Discard 3 cards")[1], Ink::Attention);
    }

    #[test]
    fn tooltip_follows_the_original_anchor_rule() {
        let size = Vec2::new(250.0, 150.0);
        let half = Vec2::new(60.0, 80.0);
        let window = Vec2::new(1460.0, 839.5);
        // Hand card near the bottom: centered above it.
        assert_eq!(
            position(Vec2::new(700.0, 700.0), half, size, window, TipAnchor::Auto),
            Vec2::new(575.0, 460.0)
        );
        // Joker at the top of the room: below it.
        assert_eq!(
            position(Vec2::new(700.0, 100.0), half, size, window, TipAnchor::Auto),
            Vec2::new(575.0, 190.0)
        );
        // Shop offer: left of it, flipped right at the left edge.
        assert_eq!(
            position(Vec2::new(700.0, 400.0), half, size, window, TipAnchor::Left),
            Vec2::new(380.0, 325.0)
        );
        assert_eq!(
            position(Vec2::new(100.0, 400.0), half, size, window, TipAnchor::Left),
            Vec2::new(170.0, 325.0)
        );
        // Always clamped inside the window.
        assert_eq!(
            position(Vec2::ZERO, half, size, Vec2::splat(100.0), TipAnchor::Auto),
            Vec2::ZERO
        );
    }

    #[test]
    fn currently_lines_match_the_original_format() {
        assert_eq!(currently_text(CurrentValue::Mult(4)), "(Currently +4 Mult)");
        assert_eq!(
            currently_text(CurrentValue::Chips(30)),
            "(Currently +30 Chips)"
        );
        assert_eq!(
            currently_text(CurrentValue::XMult(150)),
            "(Currently X1.5 Mult)"
        );
        assert_eq!(
            currently_text(CurrentValue::XMult(125)),
            "(Currently X1.25 Mult)"
        );
        assert_eq!(
            currently_text(CurrentValue::XMult(300)),
            "(Currently X3 Mult)"
        );
        assert_eq!(currently_text(CurrentValue::Money(6)), "(Currently $6)");
        assert_eq!(currently_text(CurrentValue::Count(12)), "(Currently 12)");
        let line = currently_text(CurrentValue::Mult(4));
        assert_eq!(inks(&line), vec![Ink::Base, Ink::Mult, Ink::Base]);
    }
}

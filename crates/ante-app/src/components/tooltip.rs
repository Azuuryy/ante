use super::{
    info::Info,
    pixel_rect::PixelBox,
    theme::MUTED,
    tiles::{BLACK, TILE, darken, lighten, text_px, tiles},
    widgets::{label, text_shadow},
};
use ante_core::contract::CurrentValue;
use bevy::{
    ecs::system::EntityCommands,
    picking::prelude::{Pickable, PointerOut, PointerOver},
    prelude::*,
    window::PrimaryWindow,
};

/// Fixed popup width, so the text wraps the same wherever the popup sits.
const TOOLTIP_WIDTH: f32 = 4.2 * TILE;
/// Space between the panel edge and its text. Wide enough that no line sits at the limit,
/// where a sub-pixel move could change the wrapping.
const PANEL_SIDE_PADDING: f32 = 12.0;
/// Side boxes (`info_queue`) hang left of the main popup, centred on it, like `align = 'cl'`.
const SIDE_WIDTH: f32 = 3.0 * TILE;
const SIDE_GAP: f32 = 4.0;
/// `G.C.JOKER_GREY`.
const JOKER_GREY: Color = Color::srgb(
    0xBF as f32 / 255.0,
    0xC7 as f32 / 255.0,
    0xD5 as f32 / 255.0,
);
/// `G.C.SECONDARY_SET.Enhanced`.
const ENHANCED: Color = Color::srgb(
    0x83 as f32 / 255.0,
    0x89 as f32 / 255.0,
    0xDD as f32 / 255.0,
);
/// `G.C.PURPLE`.
const PURPLE: Color = Color::srgb(
    0x88 as f32 / 255.0,
    0x67 as f32 / 255.0,
    0xA5 as f32 / 255.0,
);
/// Description text scale: `0.32 × DESCSCALE` (1 for m6x11plus).
const DESC_SCALE: f32 = 0.32;
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
    /// Playing cards show their name in dark text on a white box (`name_from_rows`).
    paper_name: bool,
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
            paper_name: false,
            #[cfg(feature = "dev-tools")]
            technical: Vec::new(),
        }
    }

    /// A playing card popup: the name sits on a white box.
    pub(crate) fn paper_name(mut self) -> Self {
        self.paper_name = true;
        self
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

/// A playing card popup as `generate_card_ui` builds it: the name on a white box, the chips
/// it scores and its enhancement rule. The edition and seal are badges with side boxes;
/// nothing is listed for a card without them.
pub(crate) fn playing_card(
    rank: char,
    suit: &str,
    enhancement: &str,
    edition: &str,
    seal: &str,
    debuffed: bool,
) -> Tip {
    let enhanced = super::info::enhancement(enhancement);
    let stone = enhancement == "Stone";
    let (title, mut effect) = if stone {
        ("Stone Card".to_owned(), "+50 Chips".to_owned())
    } else {
        (
            format!("{} of {suit}", rank_name(rank)),
            format!("+{} chips", rank_chips(rank)),
        )
    };
    if let Some(rule) = enhanced.as_ref().filter(|_| !stone) {
        effect.push('\n');
        effect.push_str(rule.body);
    }
    let mut tip = Tip::new(title, effect).paper_name();
    if debuffed {
        tip = tip.badge("Debuffed");
    } else if let Some(rule) = &enhanced {
        tip = tip.badge(rule.title);
    }
    if let Some(info) = super::info::edition(edition, false) {
        tip = tip.badge(info.title).info([info]);
    }
    if let Some(info) = super::info::seal(seal) {
        tip = tip.badge(info.title).info([info]);
    }
    tip
}

/// `G.P_CARDS` rank names.
fn rank_name(rank: char) -> String {
    match rank {
        'T' => "10".to_owned(),
        'J' => "Jack".to_owned(),
        'Q' => "Queen".to_owned(),
        'K' => "King".to_owned(),
        'A' => "Ace".to_owned(),
        other => other.to_string(),
    }
}

/// Base chips of a rank: its number, 10 for faces and 11 for the Ace.
fn rank_chips(rank: char) -> u32 {
    match rank {
        'A' => 11,
        'T' | 'J' | 'Q' | 'K' => 10,
        other => other.to_digit(10).unwrap_or(0),
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
        // Whole physical pixels, so text wraps the same wherever the panel sits.
        Vec2::new((pos.x - side).max(0.0), pos.y)
            .mul_add(Vec2::splat(dpi), Vec2::ZERO)
            .round()
            / dpi
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
                    // `card_h_popup`: a JOKER_GREY frame around a translucent dark body.
                    row.spawn((
                        Node {
                            width: px(width),
                            flex_shrink: 0.0,
                            padding: UiRect::all(tiles(0.05)),
                            ..default()
                        },
                        PixelBox::new(lighten(JOKER_GREY, 0.5)).emboss(0.07),
                        Pickable::IGNORE,
                    ))
                    .with_children(|frame| {
                        frame
                            .spawn((
                                Node {
                                    flex_grow: 1.0,
                                    padding: UiRect::axes(px(PANEL_SIDE_PADDING), tiles(0.07)),
                                    row_gap: tiles(0.07),
                                    align_items: AlignItems::Center,
                                    flex_direction: FlexDirection::Column,
                                    ..default()
                                },
                                PixelBox::new(darken(BLACK, 0.1).with_alpha(0.8)),
                                Pickable::IGNORE,
                            ))
                            .with_children(|panel| main_panel(panel, tip));
                    });
                });
            #[cfg(feature = "dev-tools")]
            if !tip.technical.is_empty() {
                stack
                    .spawn((
                        Node {
                            width: px(width),
                            padding: UiRect::axes(px(PANEL_SIDE_PADDING), px(6)),
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

/// One `info_tip_from_rows` box in a JOKER_GREY frame: the title on grey, then the rule
/// on a white box.
fn side_box(ui: &mut ChildSpawnerCommands, info: &Info) {
    ui.spawn((
        Node {
            width: percent(100),
            padding: UiRect::all(tiles(0.05)),
            ..default()
        },
        PixelBox::new(lighten(JOKER_GREY, 0.5)).emboss(0.05),
        Pickable::IGNORE,
    ))
    .with_children(|frame| {
        frame
            .spawn((
                Node {
                    flex_grow: 1.0,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                PixelBox::new(SIDE_PANEL),
                Pickable::IGNORE,
            ))
            .with_children(|panel| {
                panel
                    .spawn((
                        Node {
                            min_height: tiles(0.36),
                            padding: UiRect::all(tiles(0.03)),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|title| {
                        styled_text(title, info.title, text_px(DESC_SCALE), Color::WHITE, true);
                    });
                panel
                    .spawn((
                        Node {
                            width: percent(100),
                            min_height: tiles(0.4),
                            padding: UiRect::all(tiles(0.08)),
                            justify_content: JustifyContent::Center,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        PixelBox::new(PAPER),
                        Pickable::IGNORE,
                    ))
                    .with_children(|rule| {
                        effect_line(rule, info.body, TEXT_DARK, text_px(DESC_SCALE));
                    });
            });
    });
}

/// A text node with the game font size and an optional drop shadow.
fn styled_text(
    ui: &mut ChildSpawnerCommands,
    text: impl Into<String>,
    size: f32,
    color: Color,
    shadow: bool,
) {
    let mut entity = ui.spawn((
        Text::new(text.into()),
        TextFont {
            font_size: bevy::text::FontSize::Px(size),
            ..default()
        },
        TextColor(color),
        Pickable::IGNORE,
    ));
    if shadow {
        entity.insert(text_shadow());
    }
}

/// The `playing_card` name line on one row at the description scale: the rank and "of" in
/// `G.C.L_BLACK`, the suit in its colour.
fn playing_card_name(ui: &mut ChildSpawnerCommands, title: &str) {
    let size = text_px(DESC_SCALE);
    ui.spawn((
        Node {
            flex_wrap: FlexWrap::NoWrap,
            column_gap: px(size * 0.3),
            align_items: AlignItems::Center,
            ..default()
        },
        Pickable::IGNORE,
    ))
    .with_children(|row| {
        for word in title.split_whitespace() {
            let color = suit_color(&word.to_ascii_lowercase()).unwrap_or(TEXT_DARK);
            label(row, word, size, color);
        }
    });
}

/// `G.C.SUITS` colours by suit word.
fn suit_color(word: &str) -> Option<Color> {
    Some(match word {
        "heart" | "hearts" => Color::srgb(0.92, 0.06, 0.19),
        "diamond" | "diamonds" => Color::srgb(0.74, 0.32, 0.04),
        "club" | "clubs" => Color::srgb(0.00, 0.40, 0.27),
        "spade" | "spades" => Color::srgb(0.35, 0.20, 0.54),
        _ => return None,
    })
}

/// Chances read "1 / 4": "1 in 4" and "one-in-two" become a numerator over a total.
pub(crate) fn chance_fractions(line: &str) -> String {
    let number = |word: &str| -> Option<String> {
        let trimmed = word.trim_matches(|c: char| !c.is_alphanumeric());
        if !trimmed.is_empty() && trimmed.bytes().all(|c| c.is_ascii_digit()) {
            return Some(trimmed.to_owned());
        }
        let value = match trimmed.to_ascii_lowercase().as_str() {
            "one" => 1,
            "two" => 2,
            "three" => 3,
            "four" => 4,
            "five" => 5,
            "six" => 6,
            "seven" => 7,
            "eight" => 8,
            "nine" => 9,
            "ten" => 10,
            _ => return None,
        };
        Some(value.to_string())
    };
    let words: Vec<&str> = line.split_whitespace().collect();
    let mut out: Vec<String> = Vec::new();
    let mut index = 0;
    while index < words.len() {
        let word = words[index];
        // "one-in-two" in a single token.
        if let Some((left, right)) = word.split_once("-in-")
            && let (Some(a), Some(b)) = (number(left), number(right))
        {
            let tail: String = right.chars().skip_while(|c| c.is_alphanumeric()).collect();
            let lead: String = left.chars().take_while(|c| !c.is_alphanumeric()).collect();
            out.push(format!("{lead}{a} / {b}{tail}"));
            index += 1;
            continue;
        }
        if index + 2 < words.len()
            && words[index + 1].eq_ignore_ascii_case("in")
            && let (Some(a), Some(b)) = (number(word), number(words[index + 2]))
        {
            let lead: String = word.chars().take_while(|c| !c.is_alphanumeric()).collect();
            let tail: String = words[index + 2]
                .chars()
                .skip_while(|c| !c.is_alphanumeric())
                .skip_while(|c| c.is_alphanumeric())
                .collect();
            out.push(format!("{lead}{a} / {b}{tail}"));
            index += 3;
            continue;
        }
        out.push(word.to_owned());
        index += 1;
    }
    out.join(" ")
}

/// Name scale of `localize{type = 'name'}`: `0.55 - 0.004 x length`.
fn name_scale(name: &str) -> f32 {
    0.55 - 0.004 * name.chars().count() as f32
}

fn main_panel(panel: &mut ChildSpawnerCommands, tip: &Tip) {
    let name_size = text_px(name_scale(&tip.title));
    if tip.paper_name {
        panel
            .spawn((
                Node {
                    padding: UiRect::axes(tiles(0.1), tiles(0.05)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                PixelBox::new(PAPER).emboss(0.05),
                Pickable::IGNORE,
            ))
            .with_children(|name| playing_card_name(name, &tip.title));
    } else {
        panel
            .spawn((
                Node {
                    padding: UiRect::all(tiles(0.05)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|name| styled_text(name, &tip.title, name_size, Color::WHITE, true));
    }
    if !tip.effect.is_empty() || tip.currently.is_some() {
        // `desc_from_rows`: a white box of at least 2 x 0.8 tiles.
        panel
            .spawn((
                Node {
                    width: percent(100),
                    min_width: tiles(2.0),
                    min_height: tiles(0.8),
                    padding: UiRect::all(tiles(0.07)),
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: tiles(0.03),
                    ..default()
                },
                PixelBox::new(PAPER).emboss(0.05),
                Pickable::IGNORE,
            ))
            .with_children(|effect| {
                for line in tip.effect.lines() {
                    effect_line(effect, line, TEXT_DARK, text_px(DESC_SCALE));
                }
                if let Some(currently) = &tip.currently {
                    effect_line(effect, currently, TEXT_INACTIVE, text_px(DESC_SCALE));
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
                    row_gap: tiles(0.03),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|badges| {
                for (index, badge) in tip.badges.iter().enumerate() {
                    // `create_badge`; the card type badge is drawn at 1.2 times the size.
                    let scaling = if index == 0 && is_type_badge(badge) {
                        1.2
                    } else {
                        1.0
                    };
                    badges
                        .spawn((
                            Node {
                                min_width: tiles(2.0),
                                min_height: tiles(0.4 * scaling),
                                padding: UiRect::axes(tiles(0.06), tiles(0.03 * scaling)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            PixelBox::new(badge_color(badge)).emboss(0.05),
                            Pickable::IGNORE,
                        ))
                        .with_children(|pill| {
                            styled_text(pill, badge, text_px(0.33 * scaling), Color::WHITE, true);
                        });
                }
            });
    }
    for detail in &tip.details {
        label(panel, detail, text_px(0.28), MUTED);
    }
}

/// Card type badges (rarity, consumable set, voucher, booster, enhancement).
fn is_type_badge(badge: &str) -> bool {
    is_rarity_badge(badge)
        || matches!(
            badge,
            "Tarot" | "Planet" | "Spectral" | "Voucher" | "Booster" | "Debuffed"
        )
        || badge.ends_with(" Card")
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
        "Gold Seal" => crate::components::tiles::GOLD,
        "Red Seal" | "Debuffed" => crate::components::tiles::RED,
        "Blue Seal" => crate::components::tiles::BLUE,
        "Purple Seal" => PURPLE,
        badge if badge.ends_with(" Card") => ENHANCED,
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

fn effect_line(ui: &mut ChildSpawnerCommands, line: &str, base: Color, size: f32) {
    let line = chance_fractions(line);
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
            column_gap: px(size * 0.3),
            row_gap: px(size * 0.1),
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
                // Trailing punctuation stays outside the box.
                let figure = word.trim_end_matches(|c: char| !c.is_alphanumeric());
                let trailing = &word[figure.len()..];
                let display = if explicit_times {
                    figure.to_owned()
                } else {
                    format!("×{figure}")
                };
                // `{X:mult}`: a box with `r = 0.05` corners, far smaller than the panels'
                // stepped corners, so it is drawn as a plain rectangle.
                row.spawn((
                    Node {
                        column_gap: px(0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|group| {
                    group
                        .spawn((
                            Node {
                                padding: UiRect::axes(tiles(0.04), tiles(0.01)),
                                ..default()
                            },
                            BackgroundColor(color),
                            Pickable::IGNORE,
                        ))
                        .with_children(|pill| label(pill, display, size, Color::WHITE));
                    if !trailing.is_empty() {
                        label(group, trailing, size, base);
                    }
                });
            } else {
                label(row, word, size, color);
            }
        }
    });
}

fn word_ink(word: &str, key: &str, normalized: &[String], index: usize) -> Ink {
    if word.contains('$') {
        return Ink::Money;
    }
    if key.contains("chance")
        || is_chance_token(normalized, index)
        || is_fraction(normalized, index)
    {
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

/// "1 / 4": the two numbers and the slash of a chance fraction.
fn is_fraction(normalized: &[String], index: usize) -> bool {
    let at = |i: usize| normalized.get(i).map(String::as_str);
    let digits = |word: Option<&str>| {
        word.is_some_and(|w| !w.is_empty() && w.bytes().all(|c| c.is_ascii_digit()))
    };
    let raw = |i: usize| at(i).is_some_and(str::is_empty);
    // The slash normalizes to an empty word.
    (raw(index) && index > 0 && digits(at(index - 1)) && digits(at(index + 1)))
        || (digits(at(index)) && raw(index + 1) && digits(at(index + 2)))
        || (index >= 2 && digits(at(index)) && raw(index - 1) && digits(at(index - 2)))
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
mod tests;

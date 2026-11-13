use crate::{
    app::{navigation::DeckChoice, session::Click},
    components::{
        pixel_rect::PixelBox,
        theme::{INK, MUTED},
        widgets::{atlas_image, label, shadowed_label},
    },
};
use bevy::{
    picking::prelude::{
        Pickable, PointerButton, PointerClick, PointerOut, PointerOver, PointerPress,
        PointerRelease,
    },
    prelude::*,
    text::FontSize,
};

/// Balatro draws everything in tile units: 1 tile = 20 base px x TILESCALE 3.65.
pub const TILE: f32 = 73.0;
/// Virtual room: 20 x 11.5 tiles at the default TILESCALE.
pub const ROOM: Vec2 = Vec2::new(1460.0, 839.5);
/// The original fits the room plus `ROOM_PADDING_W` = 1 and `ROOM_PADDING_H` = 0.7 tiles
/// on each side to the window (`love.resize`), so the canvas is 22 x 12.9 tiles.
pub const CANVAS: Vec2 = Vec2::new(1606.0, 941.7);
/// Card size in tiles (`G.CARD_W`, `G.CARD_H`) and the drawn scale of a card (`Card.T.scale`).
pub const CARD_W: f32 = 2.4 * 35.0 / 41.0;
pub const CARD_H: f32 = 2.4 * 47.0 / 41.0;
pub const CARD_SCALE: f32 = 0.95;

/// Drawn card size in UI pixels: 0.95 x (CARD_W, CARD_H) tiles.
pub fn card_px() -> Vec2 {
    Vec2::new(CARD_WIDTH, CARD_HEIGHT)
}
/// Joker and consumable areas of `set_screen_positions`: 4.9 and 2.3 cards wide.
pub const JOKER_AREA: f32 = 4.9 * CARD_W * TILE;
pub const CONSUMABLE_AREA: f32 = 2.3 * CARD_W * TILE;
/// Drawn card width and height in UI pixels.
pub const CARD_WIDTH: f32 = CARD_W * CARD_SCALE * TILE;
pub const CARD_HEIGHT: f32 = CARD_H * CARD_SCALE * TILE;

/// Left edge of card `k` of `n` in an area `area` px wide that holds `limit` cards, as the
/// Joker, consumable and shop areas of `CardArea:align_cards` spread them.
pub fn spread_x(k: usize, n: usize, limit: usize, area: f32, card: f32) -> f32 {
    let m = n.max(limit);
    if m <= 1 {
        return (area - card) / 2.0;
    }
    let span = (m - 1) as f32;
    (area - card) * (k as f32 / span + 0.5 * (m - n) as f32 / span)
}
/// Em size of text at scale 1: the original renders fonts at `render_scale` x `FONTSCALE`
/// = 20 x 0.1 steps per scale, one room unit.
pub const FONT_BASE: f32 = TILE;

pub const RED: Color = Color::srgb(
    0xFE as f32 / 255.0,
    0x5F as f32 / 255.0,
    0x55 as f32 / 255.0,
);
pub const BLUE: Color = Color::srgb(
    0x00 as f32 / 255.0,
    0x9D as f32 / 255.0,
    0xFF as f32 / 255.0,
);
pub const ORANGE: Color = Color::srgb(
    0xFD as f32 / 255.0,
    0xA2 as f32 / 255.0,
    0x00 as f32 / 255.0,
);
pub const PALE_GREEN: Color = Color::srgb(
    0x56 as f32 / 255.0,
    0xA8 as f32 / 255.0,
    0x87 as f32 / 255.0,
);
pub const L_BLACK: Color = Color::srgb(
    0x4F as f32 / 255.0,
    0x63 as f32 / 255.0,
    0x67 as f32 / 255.0,
);
/// `G.C.GREEN`, used by Sell, Redeem, Open and Reroll.
pub const GREEN: Color = Color::srgb(
    0x4B as f32 / 255.0,
    0xC2 as f32 / 255.0,
    0x92 as f32 / 255.0,
);
/// `G.C.UI.BACKGROUND_INACTIVE`.
pub const INACTIVE: Color = Color::srgb(0.4, 0.4, 0.4);
/// `G.C.MONEY`, the colour of prices.
pub const MONEY: Color = Color::srgb(
    0xF3 as f32 / 255.0,
    0xB9 as f32 / 255.0,
    0x58 as f32 / 255.0,
);
/// `G.C.BLACK` and `G.C.GOLD`.
pub const BLACK: Color = Color::srgb(
    0x37 as f32 / 255.0,
    0x42 as f32 / 255.0,
    0x44 as f32 / 255.0,
);
pub const GOLD: Color = Color::srgb(
    0xEA as f32 / 255.0,
    0xC0 as f32 / 255.0,
    0x58 as f32 / 255.0,
);

/// `G.C.UI.TEXT_INACTIVE`.
pub const TEXT_INACTIVE: Color = Color::srgba(
    0x88 as f32 / 255.0,
    0x88 as f32 / 255.0,
    0x88 as f32 / 255.0,
    0x99 as f32 / 255.0,
);
/// `G.C.GREY`.
pub const GREY: Color = Color::srgb(
    0x5F as f32 / 255.0,
    0x73 as f32 / 255.0,
    0x77 as f32 / 255.0,
);
/// `G.C.IMPORTANT` and `G.C.UI.TRANSPARENT_DARK`.
pub const IMPORTANT: Color = Color::srgb(1.0, 0x9A as f32 / 255.0, 0.0);
pub const TRANSPARENT_DARK: Color = Color::srgba(
    0x22 as f32 / 255.0,
    0x22 as f32 / 255.0,
    0x22 as f32 / 255.0,
    0x22 as f32 / 255.0,
);

/// `mix_colours(a, b, p)`: `p` of `a` and the rest of `b`.
pub fn mix(a: Color, b: Color, p: f32) -> Color {
    let (a, b) = (a.to_srgba(), b.to_srgba());
    let m = |x: f32, y: f32| x * p + y * (1.0 - p);
    Color::srgba(
        m(a.red, b.red),
        m(a.green, b.green),
        m(a.blue, b.blue),
        m(a.alpha, b.alpha),
    )
}

/// `darken(colour, amount)`.
pub fn darken(color: Color, amount: f32) -> Color {
    shade(color, -amount)
}

/// `lighten(colour, amount)`.
pub fn lighten(color: Color, amount: f32) -> Color {
    shade(color, amount)
}

/// Lighten (positive) or darken (negative) like the original `lighten`/`darken`.
pub fn shade(color: Color, amount: f32) -> Color {
    let c = color.to_srgba();
    let mix = |v: f32| {
        if amount >= 0.0 {
            v + (1.0 - v) * amount
        } else {
            v * (1.0 + amount)
        }
    };
    Color::srgba(mix(c.red), mix(c.green), mix(c.blue), c.alpha)
}

pub fn tiles(t: f32) -> Val {
    Val::Px(t * TILE)
}

pub fn tile_px(t: f32) -> f32 {
    t * TILE
}

pub fn text_px(scale: f32) -> f32 {
    FONT_BASE * scale
}

fn scale_rgb(color: Color, factor: f32) -> Color {
    let srgba = color.to_srgba();
    Color::srgba(
        srgba.red * factor,
        srgba.green * factor,
        srgba.blue * factor,
        srgba.alpha,
    )
}

/// Balatro emboss rim = darken(colour, 0.3).
pub fn emboss(color: Color) -> Color {
    scale_rgb(color, 0.7)
}

/// Balatro hover overlay #00000055 composited over the base colour.
pub fn hover_overlay(color: Color) -> Color {
    scale_rgb(color, 1.0 - 0x55 as f32 / 255.0)
}

pub fn viewport_fit(physical_size: Vec2) -> (Vec2, f32) {
    let scale = (physical_size.x / CANVAS.x).min(physical_size.y / CANVAS.y);
    if !scale.is_finite() || scale <= 0.0 {
        return (CANVAS, 1.0);
    }
    (physical_size / scale, scale)
}

#[derive(Component, Clone, Copy)]
struct ButtonTheme {
    normal: Color,
    hovered: Color,
    pressed: Color,
}

pub fn tile_button(
    ui: &mut ChildSpawnerCommands,
    text: &str,
    color: Color,
    (w, h): (f32, f32),
    text_scale: f32,
    action: Click,
) {
    let mut node = ui.spawn((
        Node {
            width: tiles(w),
            height: tiles(h),
            flex_shrink: 0.0,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        PixelBox::new(color).shadow(),
    ));
    attach_button(&mut node, color, action);
    node.with_children(|parent| shadowed_label(parent, text, text_px(text_scale), INK));
}

fn attach_button(node: &mut bevy::ecs::system::EntityCommands, color: Color, action: Click) {
    node.insert(ButtonTheme {
        normal: color,
        hovered: hover_overlay(color),
        pressed: scale_rgb(color, 0.52),
    })
    .observe(
        move |event: On<PointerClick>, mut clicks: MessageWriter<Click>| {
            if event.button == PointerButton::Primary {
                clicks.write(action);
            }
        },
    )
    .observe(|event: On<PointerOver>, mut buttons: Query<ButtonParts>| {
        if let Ok((style, bg, pixel)) = buttons.get_mut(event.entity) {
            paint(bg, pixel, style.hovered);
        }
    })
    .observe(|event: On<PointerOut>, mut buttons: Query<ButtonParts>| {
        if let Ok((style, bg, pixel)) = buttons.get_mut(event.entity) {
            paint(bg, pixel, style.normal);
        }
    })
    .observe(|event: On<PointerPress>, mut buttons: Query<ButtonParts>| {
        if event.button == PointerButton::Primary
            && let Ok((style, bg, pixel)) = buttons.get_mut(event.entity)
        {
            paint(bg, pixel, style.pressed);
        }
    })
    .observe(
        |event: On<PointerRelease>, mut buttons: Query<ButtonParts>| {
            if event.button == PointerButton::Primary
                && let Ok((style, bg, pixel)) = buttons.get_mut(event.entity)
            {
                paint(bg, pixel, style.hovered);
            }
        },
    );
}

type ButtonParts = (
    &'static ButtonTheme,
    Option<&'static mut BackgroundColor>,
    Option<&'static mut PixelBox>,
);

/// Hover and press colour a plain node through its background, a pixel box through its fill.
fn paint(bg: Option<Mut<BackgroundColor>>, pixel: Option<Mut<PixelBox>>, color: Color) {
    if let Some(mut pixel) = pixel {
        pixel.fill = color;
    } else if let Some(mut bg) = bg {
        bg.0 = color;
    }
}

/// A card button of `G.UIDEF.use_and_sell_buttons` or `create_shop_card_ui`: rows of
/// (text, scale) parts, minimum size and padding in tiles. Without an action it is drawn
/// in the inactive colour and ignores clicks.
pub fn card_button(
    ui: &mut ChildSpawnerCommands,
    rows: &[&[(&str, f32)]],
    color: Color,
    (min_w, min_h): (f32, f32),
    padding: UiRect,
    action: Option<Click>,
) {
    let mut node = ui.spawn((
        Node {
            min_width: tiles(min_w),
            min_height: tiles(min_h),
            flex_shrink: 0.0,
            padding,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        PixelBox::new(if action.is_some() { color } else { INACTIVE }).shadow(),
    ));
    match action {
        Some(action) => attach_button(&mut node, color, action),
        None => {
            node.insert(Pickable::IGNORE);
        }
    }
    node.with_children(|button| {
        for row in rows {
            button
                .spawn((
                    Node {
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|line| {
                    for &(text, scale) in *row {
                        shadowed_label(line, text, text_px(scale), INK);
                    }
                });
        }
    });
}

pub fn screen_heading(ui: &mut ChildSpawnerCommands, text: &str) {
    label(ui, text, text_px(0.65), INK);
}

pub fn panel(ui: &mut ChildSpawnerCommands, content: impl FnOnce(&mut ChildSpawnerCommands)) {
    ui.spawn((
        Node {
            padding: UiRect::all(tiles(0.25)),
            border: UiRect::all(tiles(0.1)),
            border_radius: BorderRadius::all(tiles(0.18)),
            min_width: tiles(11.5),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: tiles(0.12),
            ..default()
        },
        BackgroundColor(L_BLACK),
        BorderColor::all(emboss(L_BLACK)),
    ))
    .with_children(content);
}

pub fn tile_optional(
    ui: &mut ChildSpawnerCommands,
    text: &str,
    color: Color,
    size: (f32, f32),
    text_scale: f32,
    action: Option<Click>,
    selected: bool,
) {
    if let Some(action) = action {
        tile_button(
            ui,
            text,
            if selected { BLUE } else { color },
            size,
            text_scale,
            action,
        );
    } else {
        let bg = if selected { BLUE } else { INACTIVE };
        ui.spawn((
            Node {
                width: tiles(size.0),
                height: tiles(size.1),
                flex_shrink: 0.0,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            PixelBox::new(bg),
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(text),
                TextFont {
                    font_size: FontSize::Px(text_px(text_scale)),
                    ..default()
                },
                TextColor(if selected { INK } else { TEXT_INACTIVE }),
                TextLayout::no_wrap(),
                Pickable::IGNORE,
            ));
        });
    }
}

pub fn deck_preview_large(
    ui: &mut ChildSpawnerCommands,
    choice: DeckChoice,
    back: Option<ImageNode>,
) {
    let color = match choice {
        DeckChoice::Red => Color::srgb(0.65, 0.18, 0.20),
        DeckChoice::Blue => Color::srgb(0.15, 0.36, 0.65),
        _ => Color::srgb(0.27, 0.29, 0.29),
    };
    ui.spawn((
        Node {
            width: tiles(2.0),
            height: tiles(2.3),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(color),
    ))
    .with_children(|card| {
        if let Some(image) = back {
            atlas_image(card, image, tile_px(1.45), tile_px(1.95));
        } else {
            label(card, "DECK", text_px(0.4), INK);
        }
    });
}

pub fn menu_button(ui: &mut ChildSpawnerCommands, text: &str, color: Color, action: Option<Click>) {
    let normal = if action.is_some() {
        color
    } else {
        Color::srgb(0.18, 0.20, 0.20)
    };
    let mut node = ui.spawn((
        Node {
            min_width: px(132),
            height: px(48),
            flex_shrink: 0.0,
            padding: UiRect::horizontal(px(12)),
            border_radius: BorderRadius::all(px(8)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(normal),
    ));
    if let Some(action) = action {
        attach_button(&mut node, color, action);
    } else {
        node.insert(Pickable::IGNORE);
    }
    node.with_children(|parent| {
        parent.spawn((
            Text::new(text),
            TextFont {
                font_size: FontSize::Px(19.0),
                ..default()
            },
            TextColor(if action.is_some() { INK } else { MUTED }),
            Pickable::IGNORE,
        ));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cards_spread_like_the_original_areas() {
        // Two Jokers in five slots sit centred where slots 2 and 3 of five would be.
        let (area, card) = (500.0, 100.0);
        assert!((spread_x(0, 2, 5, area, card) - 150.0).abs() < 1e-3);
        assert!((spread_x(1, 2, 5, area, card) - 250.0).abs() < 1e-3);
        // A full row spans the area; overflow overlaps.
        assert!((spread_x(4, 5, 5, area, card) - 400.0).abs() < 1e-3);
        assert!((spread_x(5, 6, 5, area, card) - 400.0).abs() < 1e-3);
        assert!((spread_x(0, 1, 1, area, card) - 200.0).abs() < 1e-3);
    }

    #[test]
    fn viewport_is_covered_without_distortion() {
        for window in [CANVAS, Vec2::new(1080.0, 700.0), Vec2::new(3440.0, 1440.0)] {
            let (canvas, scale) = viewport_fit(window);
            assert!((canvas * scale - window).length() < 1e-3);
            assert!((canvas + Vec2::splat(1e-3)).cmpge(CANVAS).all());
        }
    }

    #[test]
    fn the_room_unit_matches_the_original_at_1080p() {
        // love.resize: 1056 px of height hold 12.9 tiles, about 81.9 px per tile.
        let (_, scale) = viewport_fit(Vec2::new(1920.0, 1056.0));
        assert!((TILE * scale - 1056.0 / 12.9).abs() < 0.1);
        let card = card_px();
        assert!((card.x - 142.1).abs() < 0.1 && (card.y - 190.8).abs() < 0.1);
    }

    #[test]
    fn fit_of_zero_window_is_finite() {
        let (canvas, scale) = viewport_fit(Vec2::ZERO);
        assert_eq!(canvas, CANVAS);
        assert_eq!(scale, 1.0);
    }

    #[test]
    fn tile_and_text_helpers() {
        assert_eq!(tiles(1.0), Val::Px(73.0));
        assert_eq!(tile_px(2.0), 146.0);
        // Text at scale 1 has an em of one room unit.
        assert!((text_px(0.9) - 65.7).abs() < 1e-3);
    }

    #[test]
    fn pointer_feedback_changes_only_button_color() {
        use bevy::{
            camera::NormalizedRenderTarget,
            picking::{
                backend::HitData,
                events::Pointer,
                pointer::{Location, PointerId},
            },
        };

        let mut world = World::new();
        world.init_resource::<Messages<Click>>();
        let style = Node {
            width: px(100),
            height: px(50),
            ..default()
        };
        let mut commands = world.commands();
        let mut button = commands.spawn((style.clone(), BackgroundColor(RED)));
        attach_button(&mut button, RED, Click::Quit);
        let entity = button.id();
        world.flush();

        let pointer = Pointer::new(
            PointerId::Mouse,
            Location {
                target: NormalizedRenderTarget::None {
                    width: 100,
                    height: 50,
                },
                position: Vec2::ZERO,
            },
        );
        let hit = HitData::new(Entity::PLACEHOLDER, 0.0, None, None);
        world.trigger(PointerOver {
            entity,
            pointer: pointer.clone(),
            hit: hit.clone(),
        });
        assert_eq!(
            world.get::<BackgroundColor>(entity).unwrap().0,
            hover_overlay(RED)
        );
        world.trigger(PointerPress {
            entity,
            pointer: pointer.clone(),
            button: PointerButton::Primary,
            hit: hit.clone(),
            count: 1,
        });
        assert_eq!(
            world.get::<BackgroundColor>(entity).unwrap().0,
            scale_rgb(RED, 0.52)
        );
        assert_eq!(world.get::<Node>(entity), Some(&style));
        assert_eq!(
            world.get::<UiTransform>(entity),
            Some(&UiTransform::IDENTITY)
        );
        world.trigger(PointerOut {
            entity,
            pointer,
            hit,
        });
        assert_eq!(world.get::<BackgroundColor>(entity).unwrap().0, RED);
    }

    #[test]
    fn colour_helpers_multiply_channels() {
        let base = Color::srgb(1.0, 0.5, 0.0);
        let rim = emboss(base).to_srgba();
        assert!((rim.red - 0.7).abs() < 1e-6);
        assert!((rim.green - 0.35).abs() < 1e-6);
        let hover = hover_overlay(base).to_srgba();
        let factor = 1.0 - 0x55 as f32 / 255.0;
        assert!((hover.red - factor).abs() < 1e-6);
        assert!((hover.alpha - 1.0).abs() < 1e-6);
    }
}

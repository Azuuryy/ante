use crate::{
    app::{navigation::DeckChoice, session::Click},
    components::{
        theme::{INK, MUTED},
        widgets::{atlas_image, label},
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
/// m6x11plus design size (16 px) x TILESCALE; tunable, not pinned by source facts.
pub const FONT_BASE: f32 = 58.4;

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
    let scale = (physical_size.x / ROOM.x).min(physical_size.y / ROOM.y);
    if !scale.is_finite() || scale <= 0.0 {
        return (ROOM, 1.0);
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
            border_radius: BorderRadius::all(tiles(0.1)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(color),
    ));
    attach_button(&mut node, color, action);
    node.with_children(|parent| label(parent, text, text_px(text_scale), INK));
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
    .observe(
        |event: On<PointerOver>, mut buttons: Query<(&ButtonTheme, &mut BackgroundColor)>| {
            if let Ok((style, mut bg)) = buttons.get_mut(event.entity) {
                bg.0 = style.hovered;
            }
        },
    )
    .observe(
        |event: On<PointerOut>, mut buttons: Query<(&ButtonTheme, &mut BackgroundColor)>| {
            if let Ok((style, mut bg)) = buttons.get_mut(event.entity) {
                bg.0 = style.normal;
            }
        },
    )
    .observe(
        |event: On<PointerPress>, mut buttons: Query<(&ButtonTheme, &mut BackgroundColor)>| {
            if event.button == PointerButton::Primary
                && let Ok((style, mut bg)) = buttons.get_mut(event.entity)
            {
                bg.0 = style.pressed;
            }
        },
    )
    .observe(
        |event: On<PointerRelease>, mut buttons: Query<(&ButtonTheme, &mut BackgroundColor)>| {
            if event.button == PointerButton::Primary
                && let Ok((style, mut bg)) = buttons.get_mut(event.entity)
            {
                bg.0 = style.hovered;
            }
        },
    );
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
        let bg = if selected {
            BLUE
        } else {
            Color::srgb(0.20, 0.24, 0.24)
        };
        ui.spawn((
            Node {
                width: tiles(size.0),
                height: tiles(size.1),
                border: UiRect::all(tiles(0.08)),
                border_radius: BorderRadius::all(tiles(0.1)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(bg),
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            label(
                parent,
                text,
                text_px(text_scale),
                if selected { INK } else { MUTED },
            );
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
    fn viewport_is_covered_without_distortion() {
        for window in [ROOM, Vec2::new(1080.0, 700.0), Vec2::new(3440.0, 1440.0)] {
            let (canvas, scale) = viewport_fit(window);
            assert!((canvas * scale - window).length() < 1e-3);
            assert!((canvas + Vec2::splat(1e-3)).cmpge(ROOM).all());
        }
    }

    #[test]
    fn fit_of_zero_window_is_finite() {
        let (canvas, scale) = viewport_fit(Vec2::ZERO);
        assert_eq!(canvas, ROOM);
        assert_eq!(scale, 1.0);
    }

    #[test]
    fn tile_and_text_helpers() {
        assert_eq!(tiles(1.0), Val::Px(73.0));
        assert_eq!(tile_px(2.0), 146.0);
        assert!((text_px(0.9) - 52.56).abs() < 1e-3);
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

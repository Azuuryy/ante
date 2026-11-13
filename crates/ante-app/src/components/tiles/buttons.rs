use super::*;

#[derive(Component, Clone, Copy)]
pub(super) struct ButtonTheme {
    pub(super) normal: Color,
    pub(super) hovered: Color,
    pub(super) pressed: Color,
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

pub(super) fn attach_button(
    node: &mut bevy::ecs::system::EntityCommands,
    color: Color,
    action: Click,
) {
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

pub(super) type ButtonParts = (
    &'static ButtonTheme,
    Option<&'static mut BackgroundColor>,
    Option<&'static mut PixelBox>,
);

/// Hover and press colour a plain node through its background, a pixel box through its fill.
pub(super) fn paint(bg: Option<Mut<BackgroundColor>>, pixel: Option<Mut<PixelBox>>, color: Color) {
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

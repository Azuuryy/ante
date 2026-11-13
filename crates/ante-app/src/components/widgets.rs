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

mod card_face;
mod hover;

pub(crate) use card_face::*;
pub(crate) use hover::*;

pub(crate) fn click_target(entity: &mut bevy::ecs::system::EntityCommands, action: Click) {
    entity.observe(
        move |event: On<PointerClick>, mut clicks: MessageWriter<Click>| {
            if event.button == PointerButton::Primary {
                clicks.write(action);
            }
        },
    );
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

#[cfg(test)]
mod tests;

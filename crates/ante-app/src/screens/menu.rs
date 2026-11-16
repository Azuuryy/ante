use crate::{
    app::session::Click,
    components::{
        theme::{INK, MUTED},
        tiles::{
            BLUE, L_BLACK, ORANGE, PALE_GREEN, RED, TILE, emboss, text_px, tile_button, tile_px,
            tiles,
        },
        widgets::label,
    },
    services::assets::catalog::Atlases,
};
use bevy::{picking::prelude::Pickable, prelude::*};

const MENU_ROW_Y: f32 = 87.0;

/// The main menu; `splash` says the `splash` shader already fills the canvas behind it, so
/// the still swirl is only drawn as its fallback.
pub fn main_menu(ui: &mut ChildSpawnerCommands, atlases: &Atlases, splash: bool) {
    let full_room = Node {
        position_type: PositionType::Absolute,
        left: px(0),
        top: px(0),
        width: percent(100),
        height: percent(100),
        ..default()
    };
    match atlases.menu_swirl() {
        _ if splash => {}
        Some(mut swirl) => {
            swirl.color = Color::srgb(0.16, 0.17, 0.22);
            ui.spawn((full_room.clone(), swirl, Pickable::IGNORE));
        }
        None => {
            ui.spawn((
                full_room.clone(),
                BackgroundColor(Color::srgb(0.07, 0.08, 0.11)),
                Pickable::IGNORE,
            ));
        }
    }

    ui.spawn((
        Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..full_room
        },
        UiTransform::from_translation(Val2::px(0.0, -tile_px(1.2))),
        Pickable::IGNORE,
    ))
    .with_children(|anchor| {
        if let (Some(base), Some(face)) = (atlases.card_base(), atlases.card('A', "Spades")) {
            let ace = Vec2::new(1.2 * (2.4 * 35.0 / 41.0), 1.2 * (2.4 * 47.0 / 41.0)) * TILE;
            anchor
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(50),
                        top: percent(50),
                        width: px(ace.x),
                        height: px(ace.y),
                        ..default()
                    },
                    UiTransform::from_translation(Val2::px(-ace.x * 0.5, -ace.y * 0.5)),
                    Pickable::IGNORE,
                ))
                .with_children(|layers| {
                    for image in [base, face] {
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
                });
        }
        if let Some(logo) = &atlases.logo {
            anchor.spawn((
                ImageNode::new(logo.clone()),
                Node {
                    width: tiles(14.3),
                    aspect_ratio: Some(333.0 / 216.0),
                    ..default()
                },
                Pickable::IGNORE,
            ));
        } else {
            label(anchor, "ANTE", 110.0, INK);
        }
    });

    ui.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: percent(MENU_ROW_Y),
            width: percent(100),
            justify_content: JustifyContent::Center,
            ..default()
        },
        Pickable::IGNORE,
    ))
    .with_children(|anchor| {
        anchor
            .spawn((
                Node {
                    align_items: AlignItems::Center,
                    column_gap: tiles(0.1),
                    padding: UiRect::all(tiles(0.2)),
                    border: UiRect::all(tiles(0.1)),
                    border_radius: BorderRadius::all(tiles(0.1)),
                    ..default()
                },
                BackgroundColor(L_BLACK),
                BorderColor::all(emboss(L_BLACK)),
                UiTransform::from_translation(Val2::new(Val::Px(0.0), Val::Percent(-50.0))),
            ))
            .with_children(|panel| {
                tile_button(panel, "PLAY", BLUE, (3.65, 1.55), 0.9, Click::OpenDecks);
                tile_button(
                    panel,
                    "OPTIONS",
                    ORANGE,
                    (2.65, 1.35),
                    0.54,
                    Click::OpenOptions,
                );
                tile_button(panel, "QUIT", RED, (2.65, 1.35), 0.54, Click::Quit);
                tile_button(
                    panel,
                    "COLLECTION",
                    PALE_GREEN,
                    (3.65, 1.55),
                    0.675,
                    Click::Collection(0),
                );
            });
    });

    ui.spawn(Node {
        position_type: PositionType::Absolute,
        right: tiles(0.3),
        bottom: tiles(0.05),
        ..default()
    })
    .with_children(|anchor| {
        label(
            anchor,
            format!("v{}", env!("CARGO_PKG_VERSION")),
            text_px(0.3),
            Color::srgba(1.0, 1.0, 1.0, 0.8),
        );
    });

    ui.spawn(Node {
        position_type: PositionType::Absolute,
        left: tiles(0.3),
        bottom: tiles(0.05),
        flex_direction: FlexDirection::Column,
        row_gap: px(2),
        ..default()
    })
    .with_children(|anchor| {
        label(anchor, "F3 profiler  |  F4 export", text_px(0.3), MUTED);
        if let Some(status) = atlases.status {
            label(anchor, status, text_px(0.3), MUTED);
        }
    });
}

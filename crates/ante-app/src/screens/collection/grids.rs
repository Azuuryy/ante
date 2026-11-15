use super::layout::{blind_amount, number_format};
use super::*;
use crate::components::{
    tiles::{IMPORTANT, JOKER_GREY, RED, lighten},
    widgets::card_hover,
};

/// Tags per row of `create_UIBox_your_collection_tags`.
const TAGS_PER_ROW: usize = 6;

/// Blind chips per row of `create_UIBox_your_collection_blinds`.
const BLINDS_PER_ROW: usize = 5;

/// Antes listed beside the blinds.
const ANTES: usize = 16;

/// A hoverable sprite cell (`padding = 0.1`) with its tooltip.
fn sprite_cell(
    ui: &mut ChildSpawnerCommands,
    sprite: Option<ImageNode>,
    size: f32,
    tip: tooltip::Tip,
    spacer: (bool, bool),
) {
    ui.spawn(Node {
        padding: UiRect::all(px(0.1 * TILE)),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|cell| {
        let gap = |cell: &mut ChildSpawnerCommands| {
            cell.spawn(Node {
                height: px(0.2 * TILE),
                ..default()
            });
        };
        if spacer.0 {
            gap(cell);
        }
        let mut entity = cell.spawn(Node {
            width: px(size * TILE),
            height: px(size * TILE),
            ..default()
        });
        card_hover(&mut entity);
        tooltip::attach(&mut entity, tip);
        entity.with_children(|art| match sprite {
            Some(sprite) => atlas_image(art, sprite, size * TILE, size * TILE),
            None => fallback_art(art, "?", size * TILE),
        });
        if spacer.1 {
            gap(cell);
        }
    });
}

/// `create_UIBox_your_collection_tags`: the tags six to a row in one black box, the
/// undiscovered ones as the "?" tag.
pub(super) fn tags(
    ui: &mut ChildSpawnerCommands,
    profile: &Profile,
    atlases: &Atlases,
    catalog: &[Content],
) {
    generic_options(ui, Click::Back, |content| {
        unsaved_note(content, profile);
        content
            .spawn((
                Node {
                    padding: UiRect::all(px(0.1 * TILE)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                PixelBox::new(BLACK).emboss(0.05),
            ))
            .with_children(|grid| {
                for line in catalog.chunks(TAGS_PER_ROW) {
                    grid.spawn(Node::default()).with_children(|row| {
                        for tag in line {
                            let (discovered, locked) = shown(profile, ContentGroup::Tags, tag);
                            let id = if discovered {
                                tag.id.0
                            } else {
                                "tag_undiscovered"
                            };
                            sprite_cell(
                                row,
                                atlases.tag_icon(id),
                                0.8,
                                collection_tip(tag, ContentGroup::Tags, discovered, locked),
                                (false, false),
                            );
                        }
                    });
                }
            });
    });
}

/// `create_UIBox_your_collection_blinds`: the ante table (ANTE and BASE chips per ante)
/// beside the blind chips, five to a row with gaps after the first, third and fifth row.
pub(super) fn blinds(
    ui: &mut ChildSpawnerCommands,
    profile: &Profile,
    atlases: &Atlases,
    catalog: &[Content],
) {
    generic_options(ui, Click::Back, |content| {
        unsaved_note(content, profile);
        content.spawn(Node::default()).with_children(|body| {
            body.spawn((
                Node {
                    padding: UiRect::all(px(0.1 * TILE)),
                    ..default()
                },
                PixelBox::new(BLACK).emboss(0.05),
            ))
            .with_children(|frame| ante_table(frame, atlases));
            body.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|grid| {
                for (row_index, line) in catalog.chunks(BLINDS_PER_ROW).enumerate() {
                    grid.spawn(Node::default()).with_children(|row| {
                        for (column, blind) in line.iter().enumerate() {
                            let k = row_index * BLINDS_PER_ROW + column + 1;
                            let (discovered, locked) = shown(profile, ContentGroup::Blinds, blind);
                            let id = if discovered {
                                blind.id.0
                            } else {
                                "bl_undiscovered"
                            };
                            let tip = if discovered {
                                collection_tip(blind, ContentGroup::Blinds, true, locked)
                            } else {
                                tooltip::Tip::new(
                                    "Undiscovered",
                                    "Defeat this blind to discover it.",
                                )
                            };
                            sprite_cell(
                                row,
                                atlases.blind_chip(id),
                                1.3,
                                tip,
                                (matches!(k, 6 | 16 | 26), matches!(k, 5 | 15 | 25)),
                            );
                        }
                    });
                }
            });
        });
    });
}

/// The L_BLACK table of base chip requirements for antes 1 to 16.
fn ante_table(ui: &mut ChildSpawnerCommands, atlases: &Atlases) {
    ui.spawn((
        Node {
            padding: UiRect::all(px(0.1 * TILE)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        PixelBox::new(L_BLACK),
    ))
    .with_children(|table| {
        ante_row(
            table,
            |left| words(left, "ANTE", 0.4, lighten(IMPORTANT, 0.2)),
            |right| {
                words(right, "BASE", 0.4, lighten(RED, 0.2));
            },
        );
        // `1 - min(20, max(15, furthest ante)) * 0.06`; Ante does not record the
        // furthest ante yet, so the spacing and colours use a fresh profile's.
        let spacing = 1.0 - 15.0 * 0.06;
        for ante in 1..=ANTES {
            if ante > 1 {
                table.spawn(Node {
                    min_height: px(spacing * TILE),
                    ..default()
                });
            }
            ante_row(
                table,
                |left| words(left, ante.to_string(), 0.4, IMPORTANT),
                |right| {
                    if let Some(chip) = atlases.ui_cell(0, 0) {
                        atlas_image(right, chip, 0.2 * TILE, 0.2 * TILE);
                    }
                    words(right, number_format(blind_amount(ante)), 0.4, JOKER_GREY);
                },
            );
        }
    });
}

fn ante_row(
    ui: &mut ChildSpawnerCommands,
    left: impl FnOnce(&mut ChildSpawnerCommands),
    right: impl FnOnce(&mut ChildSpawnerCommands),
) {
    ui.spawn(Node {
        padding: UiRect::all(px(0.03 * TILE)),
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|row| {
        row.spawn(Node {
            min_width: px(0.7 * TILE),
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(left);
        row.spawn(Node {
            min_width: px(2.8 * TILE),
            column_gap: px(0.03 * TILE),
            justify_content: JustifyContent::End,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(right);
    });
}

use crate::{
    app::{playback::Speed, presentation::PlaybackLabel, session::Click},
    components::{
        pixel_rect::PixelBox,
        theme::INK,
        tiles as menu_ui,
        widgets::{
            CARD_HEIGHT, PLAY_AREA_RISE, button, card_control, label, row, shadowed_label,
            speed_button,
        },
    },
    screens::run::deck_preview,
    services::assets::catalog::Atlases,
};
use ante_core::contract::{Command, RunView};
use bevy::prelude::*;

/// `create_UIBox_buttons`: Play Hand and Discard (2.5 x 1.3, text 0.45) around the Sort Hand
/// box, a translucent dark box outlined in mix(WHITE, JOKER_GREY, 0.7).
fn actions_row(ui: &mut ChildSpawnerCommands, view: &RunView) {
    use menu_ui::{BLUE, ORANGE, RED, card_button, mix, text_px, tiles};
    ui.spawn(Node {
        column_gap: tiles(0.3),
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|actions| {
        let big = |ui: &mut ChildSpawnerCommands, text: &str, color, action: Option<Click>| {
            card_button(
                ui,
                &[&[(text, 0.45)]],
                color,
                (2.5, 1.3),
                UiRect::all(tiles(0.1)),
                action,
            );
        };
        big(
            actions,
            "Play Hand",
            BLUE,
            view.can_play.then_some(Click::Play(Command::Play)),
        );
        let joker_grey = Color::srgb(
            0xBF as f32 / 255.0,
            0xC7 as f32 / 255.0,
            0xD5 as f32 / 255.0,
        );
        actions
            .spawn((
                Node {
                    padding: UiRect::all(tiles(0.1)),
                    row_gap: tiles(0.05),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                PixelBox::new(menu_ui::TRANSPARENT_DARK).outline(
                    mix(Color::WHITE, joker_grey, 0.7),
                    1.5 * menu_ui::TILE / 20.0,
                ),
            ))
            .with_children(|sort| {
                shadowed_label(sort, "Sort Hand", text_px(0.36), INK);
                sort.spawn(Node {
                    padding: UiRect::all(tiles(0.1)),
                    column_gap: tiles(0.2),
                    ..default()
                })
                .with_children(|buttons| {
                    for (text, action) in [("Rank", Command::SortRank), ("Suit", Command::SortSuit)]
                    {
                        card_button(
                            buttons,
                            &[&[(text, 0.315)]],
                            ORANGE,
                            (0.9, 0.7),
                            UiRect::all(tiles(0.1)),
                            view.can_sort.then_some(Click::Play(action)),
                        );
                    }
                });
            });
        big(
            actions,
            "Discard",
            RED,
            view.can_discard.then_some(Click::Play(Command::Discard)),
        );
    });
}

pub(crate) fn playing_board(
    ui: &mut ChildSpawnerCommands,
    view: &RunView,
    atlases: &Atlases,
    playing_trace: bool,
    speed: Speed,
    preview: bool,
) {
    ui.spawn(Node {
        width: percent(100),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        row_gap: px(10),
        ..default()
    })
    .with_children(|area| {
        if preview {
            deck_preview(area, view);
        }
        hand_rows(area, view, atlases, playing_trace);
        if preview {
            return;
        }
        if playing_trace {
            area.spawn((PlaybackLabel, Text::new("Scoring..."), TextColor(INK)));
            row(area, |actions| {
                button(actions, "Skip", Some(Click::Skip));
                speed_button(actions, speed);
            });
        } else {
            label(area, format!("{} in hand", view.cards.len()), 18.0, INK);
            actions_row(area, view);
        }
    });
}

/// The hand, and during a scoring trace the played cards above it.
pub(crate) fn hand_rows(
    area: &mut ChildSpawnerCommands,
    view: &RunView,
    atlases: &Atlases,
    playing_trace: bool,
) {
    let groups: Vec<Vec<_>> = if playing_trace {
        vec![
            view.cards.iter().filter(|card| card.selected).collect(),
            view.cards.iter().filter(|card| !card.selected).collect(),
        ]
    } else {
        vec![view.cards.iter().collect()]
    };
    for (index, cards) in groups.into_iter().enumerate() {
        if cards.is_empty() {
            continue;
        }
        let played = playing_trace && index == 0;
        area.spawn(Node {
            justify_content: JustifyContent::Center,
            // The play area sits 3.6 room units above the hand, top to top.
            margin: UiRect::bottom(px(if played {
                PLAY_AREA_RISE - CARD_HEIGHT - 10.0
            } else {
                0.0
            })),
            ..default()
        })
        .with_children(|hand| {
            for (position, card) in cards.iter().enumerate() {
                let fg = if card.red {
                    Color::srgb(0.57, 0.12, 0.15)
                } else {
                    Color::srgb(0.10, 0.16, 0.17)
                };
                let rank = if card.rank == 'T' {
                    "10".to_owned()
                } else {
                    card.rank.to_string()
                };
                // Debug builds keep every attribute in the technical panel.
                let tip = crate::components::tooltip::playing_card(
                    card.rank,
                    card.suit,
                    &card.enhancement,
                    &card.edition,
                    &card.seal,
                    card.debuffed,
                )
                .technical(format!(
                    "Base rank: {} chips · enhancement {} · edition {} · seal {}",
                    card.chips, card.enhancement, card.edition, card.seal
                ))
                .technical(card.effect_summary.clone());
                card_control(
                    hand,
                    (card.enabled && !playing_trace)
                        .then_some(Click::Play(Command::ToggleCard(card.id))),
                    card,
                    tip,
                    (cards.len(), position),
                    (!playing_trace, played),
                    |content| {
                        let face = crate::components::widgets::PlayingFace {
                            rank: card.rank,
                            suit: card.suit,
                            enhancement: &card.enhancement,
                            edition: &card.edition,
                            seal: &card.seal,
                            debuffed: card.debuffed,
                            seed: card.id.0,
                        };
                        if !crate::components::widgets::playing_card_face(
                            content,
                            atlases,
                            &face,
                            crate::components::widgets::CARD_WIDTH - 4.0,
                        ) {
                            label(content, rank, 30.0, fg);
                            label(content, card.suit, 15.0, fg);
                        }
                    },
                );
            }
        });
    }
}

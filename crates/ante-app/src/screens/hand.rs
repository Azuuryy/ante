use crate::{
    app::{playback::Speed, presentation::PlaybackLabel, session::Click},
    components::{
        theme::INK,
        tiles as menu_ui,
        widgets::{
            CARD_HEIGHT, PLAY_AREA_RISE, button, card_control, card_face_with_seal, label, row,
            speed_button,
        },
    },
    screens::run::deck_preview,
    services::assets::catalog::Atlases,
};
use ante_core::contract::{Command as MvpAction, RunView};
use bevy::prelude::*;

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
                    let mut text = format!(
                        "{} of {}\nBase rank: {} chips\nEnhancement: {}\nEdition: {}\nSeal: {}{}",
                        rank,
                        card.suit,
                        card.chips,
                        card.enhancement,
                        card.edition,
                        card.seal,
                        if card.debuffed { "\nDebuffed" } else { "" }
                    );
                    if !card.effect_summary.is_empty() {
                        text.push('\n');
                        text.push_str(&card.effect_summary);
                    }
                    card_control(
                        hand,
                        (card.enabled && !playing_trace)
                            .then_some(Click::Play(MvpAction::ToggleCard(card.id))),
                        card,
                        text,
                        (cards.len(), position),
                        (!playing_trace, played),
                        |content| {
                            if let Some(face) = atlases.card(card.rank, card.suit) {
                                card_face_with_seal(
                                    content,
                                    atlases.card_base(),
                                    face,
                                    card.debuffed,
                                    116.0,
                                    Some((&card.seal, atlases.seal(&card.seal))),
                                    (&card.edition, card.id.0),
                                );
                            } else {
                                label(content, rank, 30.0, fg);
                                label(content, card.suit, 15.0, fg);
                            }
                        },
                    );
                }
            });
        }
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
            row(area, |actions| {
                use menu_ui::{BLUE, ORANGE, RED, tile_optional};
                tile_optional(
                    actions,
                    "Play Hand",
                    BLUE,
                    (2.7, 1.08),
                    0.49,
                    view.can_play.then_some(Click::Play(MvpAction::Play)),
                    false,
                );
                actions
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: px(4),
                        ..default()
                    })
                    .with_children(|sort| {
                        label(sort, "Sort Hand", 19.0, INK);
                        row(sort, |buttons| {
                            tile_optional(
                                buttons,
                                "Rank",
                                ORANGE,
                                (1.02, 0.61),
                                0.37,
                                view.can_sort.then_some(Click::Play(MvpAction::SortRank)),
                                false,
                            );
                            tile_optional(
                                buttons,
                                "Suit",
                                ORANGE,
                                (1.02, 0.61),
                                0.37,
                                view.can_sort.then_some(Click::Play(MvpAction::SortSuit)),
                                false,
                            );
                        });
                    });
                tile_optional(
                    actions,
                    "Discard",
                    RED,
                    (2.7, 1.08),
                    0.49,
                    view.can_discard.then_some(Click::Play(MvpAction::Discard)),
                    false,
                );
            });
        }
    });
}

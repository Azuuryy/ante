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

/// A Play Hand, Sort or Discard button. The row outlives selection changes: only this
/// state is repainted, so the buttons are not rebuilt (and do not settle again) each time a
/// card is selected, as the original keeps `G.buttons` for the whole round.
#[derive(Component, Clone, Copy, PartialEq)]
pub(crate) struct HandAction {
    color: Color,
    enabled: bool,
}

/// Paints a hand action in its colour or greyed out, and lets it take clicks only when
/// enabled.
pub(crate) fn paint_hand_actions(
    mut commands: Commands,
    mut buttons: Query<(Entity, &HandAction, &mut PixelBox), Changed<HandAction>>,
) {
    for (entity, action, mut pixel) in &mut buttons {
        let fill = if action.enabled {
            action.color
        } else {
            menu_ui::INACTIVE
        };
        if pixel.fill != fill {
            pixel.fill = fill;
        }
        commands.entity(entity).insert(if action.enabled {
            Pickable::default()
        } else {
            Pickable::IGNORE
        });
    }
}

/// Which hand actions are allowed: Play Hand, Rank, Suit, Discard.
fn allowed(view: &RunView) -> [bool; 4] {
    [
        view.can_play,
        view.can_sort,
        view.can_sort,
        view.can_discard,
    ]
}

fn in_hand(view: &RunView) -> String {
    format!("{} in hand", view.cards.len())
}

/// The parts of a steady hand (no scoring trace, no deck preview) that are updated in place.
pub(crate) struct HandSlots {
    /// The node holding the hand rows, rebuilt on each change.
    hand: Entity,
    label: Entity,
    buttons: [(Entity, Color); 4],
    allowed: [bool; 4],
}

impl HandSlots {
    /// Rebuilds the hand and repaints only the actions whose state changed.
    pub(crate) fn refresh(&mut self, commands: &mut Commands, view: &RunView, atlases: &Atlases) {
        commands
            .entity(self.hand)
            .despawn_children()
            .with_children(|area| hand_rows(area, view, atlases, false));
        commands.entity(self.label).insert(Text::new(in_hand(view)));
        let allowed = allowed(view);
        for ((&(entity, color), &enabled), before) in
            self.buttons.iter().zip(&allowed).zip(self.allowed)
        {
            if enabled != before {
                commands
                    .entity(entity)
                    .insert(HandAction { color, enabled });
            }
        }
        self.allowed = allowed;
    }
}

/// `create_UIBox_buttons`: Play Hand and Discard (2.5 x 1.3, text 0.45) around the Sort Hand
/// box, a translucent dark box outlined in mix(WHITE, JOKER_GREY, 0.7).
fn actions_row(ui: &mut ChildSpawnerCommands, view: &RunView) -> [(Entity, Color); 4] {
    use menu_ui::{BLUE, ORANGE, RED, card_button, mix, text_px, tiles};
    let allowed = allowed(view);
    let mut buttons = [(Entity::PLACEHOLDER, BLUE); 4];
    let mut button = |ui: &mut ChildSpawnerCommands,
                      slot: usize,
                      rows: &[&[(&str, f32)]],
                      color: Color,
                      size: (f32, f32),
                      command: Command| {
        let entity = card_button(
            ui,
            rows,
            color,
            size,
            UiRect::all(tiles(0.1)),
            Some(Click::Play(command)),
        );
        ui.commands().entity(entity).insert(HandAction {
            color,
            enabled: allowed[slot],
        });
        buttons[slot] = (entity, color);
    };
    ui.spawn(Node {
        column_gap: tiles(0.3),
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|actions| {
        let big = [
            ("Play Hand", BLUE, Command::Play),
            ("Discard", RED, Command::Discard),
        ];
        let (play, discard) = (big[0], big[1]);
        button(actions, 0, &[&[(play.0, 0.45)]], play.1, (2.5, 1.3), play.2);
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
                .with_children(|sorts| {
                    for (slot, text, command) in [
                        (1, "Rank", Command::SortRank),
                        (2, "Suit", Command::SortSuit),
                    ] {
                        button(
                            sorts,
                            slot,
                            &[&[(text, 0.315)]],
                            ORANGE,
                            (0.9, 0.7),
                            command,
                        );
                    }
                });
            });
        button(
            actions,
            3,
            &[&[(discard.0, 0.45)]],
            discard.1,
            (2.5, 1.3),
            discard.2,
        );
    });
    buttons
}

/// The playing board: the hand, then the actions, or the scoring controls during a trace.
/// A steady hand returns its slots so later selections update it in place.
pub(crate) fn playing_board(
    ui: &mut ChildSpawnerCommands,
    view: &RunView,
    atlases: &Atlases,
    playing_trace: bool,
    speed: Speed,
    preview: bool,
) -> Option<HandSlots> {
    let mut slots = None;
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
        let hand = area
            .spawn(Node {
                width: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(10),
                ..default()
            })
            .with_children(|hand| hand_rows(hand, view, atlases, playing_trace))
            .id();
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
            let label = area
                .spawn((
                    Text::new(in_hand(view)),
                    TextFont {
                        font_size: bevy::text::FontSize::Px(18.0),
                        ..default()
                    },
                    TextColor(INK),
                    Pickable::IGNORE,
                ))
                .id();
            slots = Some(HandSlots {
                hand,
                label,
                buttons: actions_row(area, view),
                allowed: allowed(view),
            });
        }
    });
    slots
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
                            greyed: false,
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

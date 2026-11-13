use super::*;

pub fn deck_select(
    ui: &mut ChildSpawnerCommands,
    session: &Session,
    atlases: &Atlases,
    backend: &dyn ante_core::contract::GameBackend,
) {
    let deck = session.navigation.deck.unwrap_or(DeckChoice::Red);
    let can_continue = session
        .run
        .as_ref()
        .is_some_and(|run| !matches!(run.view().phase, Phase::Won | Phase::Lost));
    modal(ui, 690.0, 18.0, |panel| {
        row(panel, |tabs| {
            tile_optional(tabs, "New Run", RED, (2.1, 0.68), 0.46, None, true);
            tile_optional(
                tabs,
                "Continue",
                RED,
                (2.1, 0.68),
                0.46,
                can_continue.then_some(Click::ContinueRun),
                false,
            );
            tile_optional(tabs, "Challenges", RED, (2.1, 0.68), 0.46, None, false);
        });
        row(panel, |picker| {
            tile_button(picker, "<", RED, (0.55, 2.9), 0.6, Click::ShiftDeck(-1));
            picker
                .spawn(box_node(
                    Node {
                        width: px(455),
                        height: px(211),
                        padding: UiRect::all(px(12)),
                        column_gap: px(16),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    WELL,
                ))
                .with_children(|preview| {
                    if let Some(back) = atlases.deck_back(deck) {
                        atlas_image(preview, back, 126.0, 169.0);
                    } else {
                        preview
                            .spawn(box_node(
                                Node {
                                    width: px(126),
                                    height: px(169),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                if deck == DeckChoice::Blue { BLUE } else { RED },
                            ))
                            .with_children(|back| {
                                label(back, "Deck", 26.0, INK);
                            });
                    }
                    preview
                        .spawn(box_node(
                            Node {
                                flex_grow: 1.0,
                                align_self: AlignSelf::Stretch,
                                justify_content: JustifyContent::Center,
                                ..column(8.0)
                            },
                            PANEL,
                        ))
                        .with_children(|details| {
                            label(details, deck.name(), 33.0, INK);
                            description(
                                details,
                                match deck {
                                    DeckChoice::Red => "+1 discard\neach round",
                                    DeckChoice::Blue => "+1 hand\neach round",
                                    _ => "Deck unavailable\nin this build",
                                },
                                256.0,
                                105.0,
                            );
                        });
                });
            tile_button(picker, ">", RED, (0.55, 2.9), 0.6, Click::ShiftDeck(1));
        });
        row(panel, |stake| {
            tile_optional(
                stake,
                "<",
                RED,
                (0.55, 1.36),
                0.5,
                (session.stake > 0).then_some(Click::ShiftStake(-1)),
                false,
            );
            stake
                .spawn(box_node(
                    Node {
                        width: px(455),
                        height: px(99),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        column_gap: px(12),
                        ..default()
                    },
                    WELL,
                ))
                .with_children(|preview| {
                    if let Some(chip) = atlases.stake_chip(session.stake) {
                        atlas_image(preview, chip, 58.0, 58.0);
                    }
                    preview.spawn(column(4.0)).with_children(|details| {
                        label(details, STAKES[session.stake], 24.0, INK);
                        description(
                            details,
                            if session.stake == 0 {
                                "Base difficulty"
                            } else {
                                "Locked in this build"
                            },
                            334.0,
                            46.0,
                        );
                    });
                });
            tile_optional(
                stake,
                ">",
                RED,
                (0.55, 1.36),
                0.5,
                (session.stake + 1 < STAKES.len()).then_some(Click::ShiftStake(1)),
                false,
            );
        });
        if session.seeded_run {
            row(panel, |seed| {
                let value = if session.seed.is_empty() {
                    "Enter seed"
                } else {
                    &session.seed
                };
                let value = if session.editing_seed {
                    format!("{value} _")
                } else {
                    value.into()
                };
                tile_button(seed, &value, BLUE, (5.0, 0.7), 0.37, Click::EditSeed);
                tile_button(seed, "Paste", BLUE, (1.65, 0.7), 0.39, Click::PasteSeed);
            });
        }
        row(panel, |actions| {
            let toggle = if session.seeded_run {
                "Seeded Run [x]"
            } else {
                "Seeded Run [ ]"
            };
            tile_button(
                actions,
                toggle,
                PANEL,
                (2.8, 0.82),
                0.34,
                Click::ToggleSeeded,
            );
            let can_start = deck.deck().is_some()
                && session.stake == 0
                && (!session.seeded_run || parse_seed(backend, &session.seed).is_some());
            tile_optional(
                actions,
                "PLAY",
                BLUE,
                (4.2, 0.96),
                0.73,
                can_start.then_some(Click::Start),
                false,
            );
        });
        if let Some(error) = &session.error {
            label(panel, error, 22.0, GOLD);
        }
        tile_button(panel, "Back", ORANGE, (8.8, 0.58), 0.46, Click::Back);
    });
}

pub(super) fn description(ui: &mut ChildSpawnerCommands, text: &str, width: f32, height: f32) {
    ui.spawn(box_node(
        Node {
            width: px(width),
            height: px(height),
            padding: UiRect::all(px(8)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        CREAM,
    ))
    .with_children(|body| {
        label(body, text, 23.0, WELL);
    });
}

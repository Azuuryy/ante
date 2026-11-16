use super::*;

pub(super) fn blind_box_node(display: Display) -> Node {
    Node {
        display,
        min_width: px(4.5 * TILE),
        padding: UiRect::all(px(0.05 * TILE)),
        ..column(0.05 * TILE)
    }
}

/// `scale_number`: text shrinks as the number grows past `max`.
pub(super) fn scale_number(number: f64, scale: f32, max: f64) -> f32 {
    if number < max {
        scale
    } else {
        (scale as f64 * (max / number).powf(0.25)).max(0.25) as f32
    }
}

/// `create_UIBox_HUD_blind`: the blind name in DYN_UI.MAIN over a DYN_UI.DARK box with the
/// rule, the blind chip and a black box with the target and reward.
pub(super) fn spawn_blind_box(
    ui: &mut ChildSpawnerCommands,
    stake: Option<ImageNode>,
    chip: &mut Entity,
    texts: &mut [Entity; 4],
) -> Entity {
    ui.spawn((
        blind_box_node(Display::None),
        PixelBox::new(BLACK).emboss(0.05),
    ))
    .with_children(|blind| {
        blind
            .spawn((
                Node {
                    width: percent(100),
                    min_height: px(0.7 * TILE),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                PixelBox::new(BLACK).emboss(0.05),
                DynRole::Main,
            ))
            .with_children(|name| {
                texts[0] = name
                    .spawn((
                        Text::new(""),
                        TextFont {
                            font_size: bevy::text::FontSize::Px(text_px(0.64)),
                            ..default()
                        },
                        TextColor(INK),
                        text_shadow(),
                        Floating,
                        UiTransform::IDENTITY,
                        Pickable::IGNORE,
                    ))
                    .id();
            });
        blind
            .spawn((
                Node {
                    width: percent(100),
                    min_height: px(2.74 * TILE),
                    justify_content: JustifyContent::Center,
                    ..column(0.0)
                },
                PixelBox::new(BLACK),
                DynRole::Dark,
            ))
            .with_children(|body| {
                texts[1] = body
                    .spawn((
                        Node {
                            max_width: px(4.2 * TILE),
                            padding: UiRect::all(px(0.05 * TILE)),
                            ..default()
                        },
                        Text::new(""),
                        TextFont {
                            font_size: bevy::text::FontSize::Px(text_px(0.36)),
                            ..default()
                        },
                        TextLayout::justify(Justify::Center),
                        TextColor(INK),
                        Pickable::IGNORE,
                    ))
                    .id();
                body.spawn(Node {
                    padding: UiRect::all(px(0.15 * TILE)),
                    column_gap: px(0.15 * TILE),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    *chip = row
                        .spawn((
                            Node {
                                width: px(1.5 * TILE),
                                height: px(1.5 * TILE),
                                ..default()
                            },
                            Pickable::IGNORE,
                        ))
                        .id();
                    row.spawn((
                        Node {
                            min_width: px(2.9 * TILE),
                            padding: UiRect::all(px(0.05 * TILE)),
                            ..column(0.0)
                        },
                        PixelBox::new(BLACK).emboss(0.05),
                    ))
                    .with_children(|target| {
                        target.spawn((
                            Text::new(""),
                            TextFont {
                                font_size: bevy::text::FontSize::Px(text_px(0.3)),
                                ..default()
                            },
                            TextColor(INK),
                            text_shadow(),
                            LocalText::new("ph_blind_score_at_least"),
                            Pickable::IGNORE,
                        ));
                        target
                            .spawn(Node {
                                min_height: px(0.6 * TILE),
                                column_gap: px(0.1 * TILE),
                                align_items: AlignItems::Center,
                                ..default()
                            })
                            .with_children(|count| {
                                if let Some(stake) = stake {
                                    atlas_image(count, stake, 0.5 * TILE, 0.5 * TILE);
                                }
                                texts[2] = value(count, text_px(0.7), RED);
                            });
                        target
                            .spawn(Node {
                                min_height: px(0.45 * TILE),
                                align_items: AlignItems::Center,
                                ..default()
                            })
                            .with_children(|reward| {
                                reward.spawn((
                                    Text::new(""),
                                    TextFont {
                                        font_size: bevy::text::FontSize::Px(text_px(0.3)),
                                        ..default()
                                    },
                                    TextColor(INK),
                                    LocalText::new("ph_blind_reward"),
                                    Pickable::IGNORE,
                                ));
                                texts[3] = value(reward, text_px(0.45), MONEY);
                            });
                    });
                });
            });
    })
    .id()
}

pub(super) fn tag_icon(
    ui: &mut ChildSpawnerCommands,
    atlases: &Atlases,
    tag: &ante_core::contract::TagView,
) {
    let mut icon = ui.spawn(Node {
        width: px(43),
        height: px(43),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    });
    crate::components::tooltip::attach(
        &mut icon,
        // The tag popup has no type badge (`Tag:get_uibox_table`).
        crate::components::tooltip::Tip::new(&tag.name, &tag.content.description)
            .info(crate::components::info::references(tag.content.id.0))
            .technical(tag.content.id.0),
    );
    icon.with_children(|art| {
        if let Some(image) = atlases.tag_icon(tag.content.id.0) {
            atlas_image(art, image, 43.0, 43.0);
        } else {
            label(art, "Tag", 19.0, GOLD);
        }
    });
}

/// Where a blind panel is drawn: the blind choice, or the Blinds tab of Run Info
/// (`create_UIBox_blind_choice(type, run_info)`), which shows the blind's state instead of
/// its buttons.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum BlindPanel {
    Choice { revision: u64 },
    RunInfo { current: bool },
}

/// The state `run_info` prints on a blind, with its colour: Current in RED, Defeated in
/// GREY, Skipped in BLUE, Upcoming in ORANGE and anything else (Select) in GOLD.
pub(super) fn run_info_state(blind: &BlindView, current: bool) -> (&'static str, Color) {
    use crate::components::tiles::GREY;
    if current {
        ("Current", RED)
    } else if blind.skipped {
        ("Skipped", BLUE)
    } else if blind.passed {
        ("Defeated", GREY)
    } else if blind.selectable {
        ("Select", GOLD)
    } else {
        ("Upcoming", ORANGE)
    }
}

pub(super) fn blind_panel(
    ui: &mut ChildSpawnerCommands,
    blind: &BlindView,
    index: usize,
    atlases: &Atlases,
    mode: BlindPanel,
) {
    let (revision, run_info) = match mode {
        BlindPanel::Choice { revision } => (revision, None),
        BlindPanel::RunInfo { current } => (0, Some(current)),
    };
    // In Run Info the panels stand level, and the current or selectable one stays lit.
    let lit = blind.selectable || run_info == Some(true);
    let color = match index {
        0 => BLUE,
        1 => GOLD,
        _ => Color::srgb(0.63, 0.63, 0.55),
    };
    let muted = crate::components::tiles::emboss(color);
    let offset = if lit || run_info.is_some() { 0.0 } else { 38.0 };
    let mut panel = ui.spawn((
        Node {
            margin: UiRect::top(px(offset)),
            flex_grow: 1.0,
            flex_basis: px(0),
            max_width: px(260),
            padding: UiRect::all(px(10)),
            border: UiRect::all(px(4)),
            border_radius: BorderRadius::all(px(15)),
            ..column(14.0)
        },
        BackgroundColor(if lit {
            PANEL
        } else {
            Color::srgb(0.16, 0.23, 0.23)
        }),
        BorderColor::all(if lit { color } else { muted }),
    ));
    // `create_UIBox_blind_choice` gives the blind no popup: its rule is printed on it, and
    // a hover would hide the tag popup of the skip button inside it.
    panel.with_children(|card| {
        if let Some(current) = run_info {
            let (state, colour) = run_info_state(blind, current);
            card.spawn((
                Node {
                    width: crate::components::tiles::tiles(2.49),
                    height: crate::components::tiles::tiles(0.64),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                PixelBox::new(colour).emboss(0.08),
                Pickable::IGNORE,
            ))
            .with_children(|state_box| label(state_box, state, text_px(0.42), INK));
        }
        let selection = if blind.selectable {
            "Select"
        } else if blind.skipped {
            "Skipped"
        } else if blind.passed {
            "Defeated"
        } else {
            "Upcoming"
        };
        if run_info.is_none() {
            tile_optional(
                card,
                selection,
                ORANGE,
                (2.49, 0.64),
                0.42,
                blind.selectable.then_some(Click::Play(Command::StartBlind)),
                false,
            );
        }
        label(
            card,
            &blind.name,
            text_px(0.45),
            if lit { INK } else { MUTED },
        );
        if let Some(mut chip) = atlases.blind_chip(blind.id) {
            if !lit {
                chip.color = Color::srgb(0.65, 0.66, 0.62);
            }
            atlas_image(card, chip, 86.0, 86.0);
        } else {
            label(card, ["Small", "Big", "Boss"][index], 28.0, color);
        }
        card.spawn(box_node(
            Node {
                width: percent(100),
                padding: UiRect::all(px(10)),
                ..column(4.0)
            },
            WELL,
        ))
        .with_children(|score| {
            if !blind.description.is_empty() {
                label(score, &blind.description, 21.0, MUTED);
            }
            label(score, "Score at least", 21.0, INK);
            label(score, score_text(blind.target), 36.0, RED);
            label(
                score,
                format!("Reward: {}", "$".repeat(blind.reward)),
                20.0,
                GOLD,
            );
        });
        if let Some(tag) = &blind.tag {
            label(card, "or", 23.0, INK);
            row(card, |skip| {
                tag_icon(skip, atlases, tag);
                if run_info.is_some() {
                    // `create_UIBox_blind_tag(type, true)`: "Skip Reward" in place of the
                    // Skip Blind button.
                    skip.spawn((
                        Node {
                            max_width: crate::components::tiles::tiles(2.0),
                            padding: UiRect::all(crate::components::tiles::tiles(0.1)),
                            ..default()
                        },
                        PixelBox::new(crate::components::tiles::mix(BLUE, BLACK, 0.4)).emboss(0.05),
                        Pickable::IGNORE,
                    ))
                    .with_children(|reward| {
                        label(reward, "Skip Reward", text_px(0.35), INK);
                    });
                } else {
                    tile_optional(
                        skip,
                        "Skip Blind",
                        RED,
                        (1.92, 0.68),
                        0.37,
                        blind
                            .can_skip
                            .then_some(Click::Play(Command::SkipBlind { revision })),
                        false,
                    );
                }
            });
            label(card, &tag.name, 20.0, MUTED);
        } else if run_info.is_none() {
            label(card, "Final blind of the ante", 20.0, GOLD);
        }
    });
}

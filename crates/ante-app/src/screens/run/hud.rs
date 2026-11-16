use super::*;

/// Mult as the original `number_format` writes it: whole numbers as is, otherwise two
/// decimals under 10, one under 100 and none above, with thousands separators.
pub fn mult_text(mult: f64) -> String {
    if mult >= 100_000_000_000.0 {
        return format!("{mult:.3e}");
    }
    let text = if mult == mult.floor() || mult >= 100.0 {
        format!("{mult:.0}")
    } else if mult >= 10.0 {
        format!("{mult:.1}")
    } else {
        format!("{mult:.2}")
    };
    let (whole, fraction) = text
        .split_once('.')
        .map_or((text.as_str(), None), |(w, f)| (w, Some(f)));
    let mut grouped = String::new();
    for (index, digit) in whole.chars().enumerate() {
        if index > 0 && (whole.len() - index) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    match fraction {
        Some(fraction) => format!("{grouped}.{fraction}"),
        None => grouped,
    }
}

pub fn score_text(score: usize) -> String {
    if score as u128 >= 100_000_000_000 {
        format!("{:.3e}", score as f64)
    } else {
        score.to_string()
    }
}

pub(super) fn value(ui: &mut ChildSpawnerCommands, size: f32, color: Color) -> Entity {
    ui.spawn((
        Text::new(""),
        TextFont {
            font_size: bevy::text::FontSize::Px(size),
            ..default()
        },
        TextColor(color),
        text_shadow(),
        Pickable::IGNORE,
    ))
    .id()
}

/// Spacing between HUD counters (`spacing` in `create_UIBox_HUD`).
pub(super) const HUD_SPACING: f32 = 0.13;

/// `contents.dollars_chips`: "Round score" beside a dark box with the stake chip and score.
pub(super) fn round_score(
    ui: &mut ChildSpawnerCommands,
    stake: Option<ImageNode>,
    score: &mut Entity,
) {
    ui.spawn((
        Node {
            padding: UiRect::all(px(0.1 * TILE)),
            column_gap: px(0.1 * TILE),
            align_items: AlignItems::Center,
            ..default()
        },
        PixelBox::new(BLACK).emboss(0.05),
        DynRole::BossMain,
    ))
    .with_children(|row| {
        row.spawn(Node {
            min_width: px(1.3 * TILE),
            ..column(0.0)
        })
        .with_children(|title| {
            shadowed_label(title, "Round", text_px(0.42), INK);
            shadowed_label(title, "score", text_px(0.42), INK);
        });
        row.spawn((
            Node {
                min_width: px(3.3 * TILE),
                min_height: px(0.7 * TILE),
                column_gap: px(0.1 * TILE),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            PixelBox::new(BLACK),
            DynRole::BossDark,
        ))
        .with_children(|count| {
            if let Some(chip) = stake {
                atlas_image(count, chip, 0.5 * TILE, 0.5 * TILE);
            }
            *score = value(count, text_px(0.85), INK);
        });
    });
}

/// `contents.hand`: the hand name and level over Chips x Mult.
pub(super) fn hand_area(
    ui: &mut ChildSpawnerCommands,
    title: &mut Entity,
    tiles: &mut [Entity; 2],
) {
    ui.spawn((
        Node {
            padding: UiRect::all(px(0.03 * TILE)),
            ..column(0.0)
        },
        PixelBox::new(darken(BLACK, 0.1)).emboss(0.05),
    ))
    .with_children(|area| {
        area.spawn(Node {
            min_height: px(1.1 * TILE),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|name| {
            *title = value(name, text_px(0.56), INK);
        });
        area.spawn(Node {
            min_height: px(TILE),
            padding: UiRect::all(px(0.1 * TILE)),
            column_gap: px(0.05 * TILE),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            for (index, color) in [BLUE, RED].into_iter().enumerate() {
                if index == 1 {
                    shadowed_label(row, "X", text_px(0.8), RED);
                }
                row.spawn((
                    Node {
                        min_width: px(2.0 * TILE),
                        min_height: px(TILE),
                        padding: UiRect::horizontal(px(0.1 * TILE)),
                        align_items: AlignItems::Center,
                        justify_content: if index == 0 {
                            JustifyContent::End
                        } else {
                            JustifyContent::Start
                        },
                        ..default()
                    },
                    PixelBox::new(color).emboss(0.05),
                ))
                .with_children(|tile| {
                    // The flame burns behind the number (`flame_chips`, `flame_mult`).
                    tile.spawn(crate::components::flames::flame_layer(index == 0));
                    tiles[index] = value(tile, text_px(0.92), INK);
                });
            }
        });
    });
}

/// `contents.buttons` and `contents.round`.
pub(super) fn bottom_hud(
    ui: &mut ChildSpawnerCommands,
    hud: &mut [Entity; 8],
    ante_end: &mut Entity,
) {
    ui.spawn(Node {
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|bottom| {
        bottom
            .spawn(Node {
                padding: UiRect::all(px(0.2 * TILE)),
                ..column(0.2 * TILE)
            })
            .with_children(|actions| {
                hud_button(actions, &["Run", "Info"], RED, Click::OpenRunInfo);
                hud_button(actions, &["Options"], ORANGE, Click::OpenPause);
            });
        bottom
            .spawn(column(HUD_SPACING * TILE))
            .with_children(|stats| {
                stats
                    .spawn(Node {
                        column_gap: px(HUD_SPACING * TILE),
                        ..default()
                    })
                    .with_children(|pair| {
                        hud[3] = counter(pair, Some("Hands"), (1.45, 0.0), BLUE, 0.8).0;
                        hud[4] = counter(pair, Some("Discards"), (1.45, 0.0), RED, 0.8).0;
                    });
                hud[5] = counter(stats, None, (1.45 * 2.0 + HUD_SPACING, 1.15), MONEY, 0.88).0;
                stats
                    .spawn(Node {
                        column_gap: px(HUD_SPACING * TILE),
                        ..default()
                    })
                    .with_children(|pair| {
                        let (ante, inner) =
                            counter(pair, Some("Ante"), (1.45, 1.0), IMPORTANT, 0.8);
                        hud[6] = ante;
                        pair.commands_mut().entity(inner).with_children(|box_| {
                            *ante_end = box_
                                .spawn((
                                    Text::new(""),
                                    TextFont {
                                        font_size: bevy::text::FontSize::Px(text_px(0.4)),
                                        ..default()
                                    },
                                    TextColor(INK),
                                    text_shadow(),
                                    Pickable::IGNORE,
                                ))
                                .id();
                        });
                        hud[7] = counter(pair, Some("Round"), (1.45, 1.0), IMPORTANT, 0.8).0;
                    });
            });
    });
}

/// Run Info and Options: 1.5 x 1.75 with a shadow.
pub(super) fn hud_button(
    ui: &mut ChildSpawnerCommands,
    lines: &[&str],
    color: Color,
    action: Click,
) {
    let rows: Vec<[(&str, f32); 1]> = lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            [(
                *line,
                if index == 0 && lines.len() > 1 {
                    0.48
                } else {
                    0.4
                },
            )]
        })
        .collect();
    let rows: Vec<&[(&str, f32)]> = rows.iter().map(|row| row.as_slice()).collect();
    crate::components::tiles::card_button(
        ui,
        &rows,
        color,
        (1.5, 1.75),
        UiRect::all(px(0.05 * TILE)),
        Some(action),
    );
}

/// A HUD counter: a BOSS_MAIN box with its title over a BOSS_DARK box holding the number.
/// Returns the number and the inner box.
pub(super) fn counter(
    ui: &mut ChildSpawnerCommands,
    title: Option<&str>,
    (min_w, min_h): (f32, f32),
    color: Color,
    scale: f32,
) -> (Entity, Entity) {
    let mut number = Entity::PLACEHOLDER;
    let mut inner = Entity::PLACEHOLDER;
    ui.spawn((
        Node {
            min_width: px(min_w * TILE),
            min_height: px(min_h * TILE),
            padding: UiRect::all(px(0.05 * TILE)),
            justify_content: JustifyContent::Center,
            ..column(0.0)
        },
        PixelBox::new(BLACK).emboss(0.05),
        DynRole::BossMain,
    ))
    .with_children(|field| {
        if let Some(title) = title {
            field
                .spawn(Node {
                    min_height: px(0.33 * TILE),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| shadowed_label(row, title, text_px(0.34), INK));
        }
        let wide = title.is_none();
        inner = field
            .spawn((
                Node {
                    min_width: px(if wide { 1.28 * 2.0 + HUD_SPACING } else { 1.2 } * TILE),
                    min_height: px(if wide { 1.0 } else { 0.0 } * TILE),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                PixelBox::new(BLACK),
                DynRole::BossDark,
            ))
            .with_children(|number_box| {
                number = value(number_box, text_px(scale), color);
            })
            .id();
    });
    (number, inner)
}

pub(super) fn center_layout(phase: Phase) -> Node {
    let mut node = Node {
        position_type: PositionType::Absolute,
        left: px(CENTER_LEFT),
        right: px(204),
        top: px(219),
        bottom: px(crate::screens::hand::CENTER_BOTTOM),
        justify_content: if phase == Phase::Playing {
            JustifyContent::End
        } else {
            JustifyContent::Center
        },
        ..column(14.0)
    };
    if phase == Phase::Shop {
        // `G.shop` aligns 'tmi' on the hand with offset −5.3; it runs below the window.
        node.top = px((11.5 - 0.95 * CARD_H - 5.3) * TILE);
        node.bottom = Val::Auto;
        node.justify_content = JustifyContent::Start;
        node.align_items = AlignItems::Center;
    }
    if phase == Phase::BeforeBlind {
        node.flex_direction = FlexDirection::Row;
        node.align_items = AlignItems::Stretch;
        node.column_gap = px(15);
    }
    node
}

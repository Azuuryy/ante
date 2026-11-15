//! `create_UIBox_round_evaluation` and `add_round_eval_row`: the panel shown once a blind is
//! beaten, whose rows, dollar signs and Cash Out button appear one after another.

use super::*;
use crate::services::assets::{
    sounds::{Sfx, jitter},
    ui_text::UiText,
};
use ante_core::contract::{RewardKind, RewardRow};

mod schedule;

use schedule::{DOLLAR_SUM_ABOVE, Reveal, schedule};

/// Panel width in tiles: the hand area less two tiles.
const PANEL_W: f32 = 6.0 * CARD_W - 2.0;

/// Dollar signs per line before a new one starts.
const DOLLARS_PER_LINE: usize = 30;

/// The panel and the moments its parts appear.
#[derive(Component)]
pub(crate) struct RoundEval {
    revision: u64,
    rows: Vec<RewardRow>,
    reveals: Vec<(f32, Reveal)>,
}

/// A part of the panel hidden until its moment.
#[derive(Component)]
pub(crate) struct RevealPart(Reveal);

/// Game time spent on the panel of one revision, kept across rebuilds of the board so a
/// redraw does not replay it, and how many reveals already played their sound.
#[derive(Resource, Default)]
pub(crate) struct RoundEvalClock {
    revision: Option<u64>,
    elapsed: f32,
    sounded: usize,
}

fn text(ui: &mut ChildSpawnerCommands, value: impl Into<String>, scale: f32, color: Color) {
    ui.spawn((
        Text::new(value),
        TextFont {
            font_size: bevy::text::FontSize::Px(text_px(scale)),
            ..default()
        },
        TextColor(color),
        text_shadow(),
        Pickable::IGNORE,
    ));
}

fn hidden(reveal: Reveal) -> (RevealPart, Visibility) {
    (RevealPart(reveal), Visibility::Hidden)
}

/// Scale of the dollar signs of a row (`add_round_eval_row`).
fn dollar_scale(dollars: usize) -> f32 {
    if dollars > 20 {
        0.28
    } else if dollars > 9 {
        0.43
    } else {
        0.58
    }
}

pub(super) fn round_eval_panel(
    ui: &mut ChildSpawnerCommands,
    view: &RunView,
    atlases: &Atlases,
    stake: usize,
    words: &UiText,
) {
    let rows = view.reward_rows.clone();
    let reveals = schedule(&rows);
    ui.spawn((
        Node {
            width: px(PANEL_W * TILE),
            min_height: px(3.0 * TILE),
            flex_shrink: 0.0,
            padding: UiRect::all(px(0.1 * TILE)),
            ..column(0.0)
        },
        PixelBox::new(BLACK).emboss(0.05),
        RoundEval {
            revision: view.revision,
            rows: rows.clone(),
            reveals,
        },
    ))
    .with_children(|panel| {
        // The Cash Out button sits in the top row once every row is counted.
        panel
            .spawn(Node {
                width: percent(100),
                min_height: px(1.4 * TILE),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|top| {
                top.spawn((Node::default(), hidden(Reveal::CashOut)))
                    .with_children(|slot| cash_out_button(slot, view.reward, words));
            });
        for (index, row) in rows.iter().enumerate() {
            if index > 0 && rows[index - 1].kind == RewardKind::Blind {
                panel
                    .spawn((
                        Node {
                            width: percent(100),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        hidden(Reveal::Divider),
                    ))
                    .with_children(|line| {
                        text(line, ".".repeat(38), 0.45, Color::WHITE);
                    });
            }
            eval_row(panel, index, row, view, atlases, stake, words);
        }
    });
}

fn cash_out_button(ui: &mut ChildSpawnerCommands, reward: usize, words: &UiText) {
    let mut button = ui.spawn((
        Node {
            min_width: px(7.0 * TILE),
            min_height: px(1.0 * TILE),
            padding: UiRect::all(px(0.1 * TILE)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        PixelBox::new(ORANGE).shadow(),
    ));
    crate::components::tiles::attach_button(&mut button, ORANGE, Click::Play(Command::CashOut));
    button.with_children(|label| {
        text(
            label,
            format!("{}: ", words.get("b_cash_out")),
            1.0,
            Color::WHITE,
        );
        text(label, format!("${reward}"), 1.08, Color::WHITE);
    });
}

fn eval_row(
    panel: &mut ChildSpawnerCommands,
    index: usize,
    row: &RewardRow,
    view: &RunView,
    atlases: &Atlases,
    stake: usize,
    words: &UiText,
) {
    panel
        .spawn(Node {
            width: percent(100),
            min_height: px(0.61 * TILE),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|line| {
            line.spawn((
                Node {
                    width: percent(55),
                    column_gap: px(0.1 * TILE),
                    align_items: AlignItems::Center,
                    ..default()
                },
                hidden(Reveal::Row(index)),
            ))
            .with_children(|left| row_label(left, row, view, atlases, stake, words));
            line.spawn(Node {
                width: percent(45),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::End,
                ..default()
            })
            .with_children(|right| dollar_signs(right, index, row.dollars));
        });
}

fn row_label(
    left: &mut ChildSpawnerCommands,
    row: &RewardRow,
    view: &RunView,
    atlases: &Atlases,
    stake: usize,
    words: &UiText,
) {
    match &row.kind {
        RewardKind::Blind => {
            if let Some(chip) = view.blind_chip.and_then(|id| atlases.blind_chip(id)) {
                atlas_image(left, chip, 1.2 * TILE, 1.2 * TILE);
            }
            left.spawn(Node {
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .with_children(|target| {
                text(target, words.get("ph_score_at_least"), 0.36, Color::WHITE);
                target
                    .spawn(Node {
                        column_gap: px(0.1 * TILE),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|score| {
                        if let Some(chip) = atlases.stake_chip(stake) {
                            atlas_image(score, chip, 0.5 * TILE, 0.5 * TILE);
                        }
                        let scale = scale_number(view.target as f64, 0.9, 100_000.0);
                        text(score, score_text(view.target), scale, RED);
                    });
            });
        }
        RewardKind::Hands { count } => {
            text(left, count.to_string(), 0.72, BLUE);
            text(
                left,
                words.with_vars("remaining_hand_money", &[1]),
                0.36,
                Color::WHITE,
            );
        }
        RewardKind::Joker { name, .. } => text(left, *name, 0.72, ORANGE),
        RewardKind::Tag(tag) => {
            if let Some(icon) = atlases.tag_icon(tag.0) {
                atlas_image(left, icon, 0.7 * TILE, 0.7 * TILE);
            }
        }
        RewardKind::Interest { max } => {
            text(left, row.dollars.to_string(), 0.72, MONEY);
            text(
                left,
                words.with_vars("interest", &[1, 5, *max]),
                0.36,
                Color::WHITE,
            );
        }
    }
}

fn dollar_signs(right: &mut ChildSpawnerCommands, row: usize, dollars: usize) {
    if dollars > DOLLAR_SUM_ABOVE {
        right
            .spawn((Node::default(), hidden(Reveal::DollarSum(row))))
            .with_children(|sum| text(sum, format!("${dollars}"), 0.6, MONEY));
        return;
    }
    let scale = dollar_scale(dollars);
    for start in (0..dollars).step_by(DOLLARS_PER_LINE) {
        right
            .spawn(Node {
                justify_content: JustifyContent::End,
                ..default()
            })
            .with_children(|line| {
                for index in start..dollars.min(start + DOLLARS_PER_LINE) {
                    line.spawn((Node::default(), hidden(Reveal::Dollar { row, index })))
                        .with_children(|sign| text(sign, "$", scale, MONEY));
                }
            });
    }
}

/// Advances the panel in game time, shows the parts whose moment has come and plays their
/// sound once: `cancel` and `highlight1` for a row, `coin3` for each dollar.
pub(crate) fn reveal_round_eval(
    time: Res<Time<Real>>,
    session: Res<Session>,
    mut clock: ResMut<RoundEvalClock>,
    panels: Query<&RoundEval>,
    mut parts: Query<(&RevealPart, &mut Visibility)>,
    mut sounds: MessageWriter<Sfx>,
) {
    let Ok(panel) = panels.single() else {
        return;
    };
    if clock.revision != Some(panel.revision) {
        *clock = RoundEvalClock {
            revision: Some(panel.revision),
            ..default()
        };
    }
    clock.elapsed += time.delta_secs() * session.speed.factor();
    let shown = panel
        .reveals
        .iter()
        .take_while(|(at, _)| *at <= clock.elapsed)
        .count();
    for (part, mut visibility) in &mut parts {
        if panel.reveals[..shown].iter().any(|(_, r)| *r == part.0) {
            visibility.set_if_neq(Visibility::Inherited);
        }
    }
    for &(_, reveal) in &panel.reveals[clock.sounded.min(shown)..shown] {
        match reveal {
            Reveal::Row(row) => {
                let pitch = 0.95 + 0.06 * row as f32;
                sounds.write(Sfx::new("cancel", pitch, 1.0));
                sounds.write(Sfx::new("highlight1", 1.5 * pitch, 0.2));
            }
            Reveal::Dollar { row, .. } | Reveal::DollarSum(row) => {
                let dollars = panel.rows.get(row).map_or(0, |r| r.dollars);
                let volume = if dollars > 20 { 0.5 } else { 0.7 };
                sounds.write(Sfx::new("coin3", jitter(0.9, 0.2), volume));
            }
            Reveal::Divider | Reveal::CashOut => {}
        }
    }
    clock.sounded = clock.sounded.max(shown);
}

#[cfg(test)]
mod tests;

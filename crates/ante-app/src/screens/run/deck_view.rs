//! `G.UIDEF.view_deck`: the deck box with its tallies, the rank counts and one `title`
//! card row per suit, inside `create_UIBox_generic_options`.

use super::*;
use crate::components::{
    card_area::{AreaKind, CardArea},
    card_row::card_row,
    tiles::{CARD_HEIGHT, CARD_WIDTH, JOKER_GREY, L_BLACK, generic_options},
};
use ante_core::contract::DeckCardView;

mod tally;

use tally::{DeckTally, Tally};

/// `G.C.UI.TEXT_DARK`.
const TEXT_DARK: Color = Color::srgb(
    0x4F as f32 / 255.0,
    0x63 as f32 / 255.0,
    0x67 as f32 / 255.0,
);

/// `tally_sprite` cells of ui_assets: Aces, faces and numbers, then each suit.
const TALLY_ICONS: [(u32, u32); 3] = [(1, 0), (2, 0), (3, 0)];
const SUIT_ICONS: [(u32, u32); 4] = [(3, 1), (0, 1), (2, 1), (1, 1)];

fn words(ui: &mut ChildSpawnerCommands, text: impl Into<String>, scale: f32, color: Color) {
    shadowed_label(ui, text, text_px(scale), color);
}

/// A count, or the base count in white beside the effective one in blue when they differ
/// (the original alternates them in one DynaText).
fn count(ui: &mut ChildSpawnerCommands, tally: Tally, scale: f32) {
    ui.spawn(Node {
        column_gap: px(0.08 * TILE),
        ..default()
    })
    .with_children(|value| {
        words(value, tally.base.to_string(), scale, Color::WHITE);
        if tally.modded() {
            words(value, tally.effective.to_string(), scale, BLUE);
        }
    });
}

/// `tally_sprite`: an 0.5-tile icon on JOKER_GREY over its count.
fn tally_sprite(ui: &mut ChildSpawnerCommands, atlases: &Atlases, icon: (u32, u32), tally: Tally) {
    ui.spawn(Node {
        padding: UiRect::all(px(0.07 * TILE)),
        ..column(0.0)
    })
    .with_children(|cell| {
        cell.spawn((
            Node {
                padding: UiRect::all(px(0.04 * TILE)),
                ..default()
            },
            PixelBox::new(JOKER_GREY).emboss(0.05),
        ))
        .with_children(|frame| match atlases.ui_cell(icon.0, icon.1) {
            Some(sprite) => atlas_image(frame, sprite, 0.5 * TILE, 0.5 * TILE),
            None => {
                frame.spawn(Node {
                    width: px(0.5 * TILE),
                    height: px(0.5 * TILE),
                    ..default()
                });
            }
        });
        count(cell, tally, 0.4);
    });
}

fn tally_row(
    ui: &mut ChildSpawnerCommands,
    atlases: &Atlases,
    cells: impl IntoIterator<Item = ((u32, u32), Tally)>,
) {
    ui.spawn(Node {
        padding: UiRect::all(px(0.1 * TILE)),
        justify_content: JustifyContent::Center,
        ..default()
    })
    .with_children(|row| {
        for (icon, tally) in cells {
            tally_sprite(row, atlases, icon, tally);
        }
    });
}

/// Deck name, its description on white, then the base (and effective) tallies.
fn deck_box(ui: &mut ChildSpawnerCommands, atlases: &Atlases, deck: DeckChoice, tally: &DeckTally) {
    ui.spawn(Node {
        padding: UiRect::all(px(0.1 * TILE)),
        ..column(0.1 * TILE)
    })
    .with_children(|info| {
        info.spawn((
            Node {
                padding: UiRect::all(px(0.15 * TILE)),
                ..column(0.1 * TILE)
            },
            PixelBox::new(L_BLACK).emboss(0.05),
        ))
        .with_children(|name| {
            let scale = 0.6 - deck.name().len() as f32 * 0.01;
            words(name, deck.name(), scale, Color::WHITE);
            name.spawn((
                Node {
                    min_width: px(2.5 * TILE),
                    min_height: px(1.3 * TILE),
                    max_width: px(3.2 * TILE),
                    padding: UiRect::all(px(0.1 * TILE)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                PixelBox::new(Color::WHITE).emboss(0.05),
            ))
            .with_children(|text| {
                text.spawn((
                    Text::new(deck.effect()),
                    TextFont {
                        font_size: bevy::text::FontSize::Px(text_px(0.32)),
                        ..default()
                    },
                    TextLayout::justify(Justify::Center),
                    TextColor(TEXT_DARK),
                    Pickable::IGNORE,
                ));
            });
        });
        info.spawn((
            Node {
                padding: UiRect::all(px(0.07 * TILE)),
                ..column(0.0)
            },
            PixelBox::new(Color::NONE).outline(L_BLACK, 1.5),
        ))
        .with_children(|tallies| {
            tallies
                .spawn(Node {
                    column_gap: px(0.1 * TILE),
                    ..default()
                })
                .with_children(|title| {
                    words(title, "Base Cards", 0.4, RED);
                    if tally.modded() {
                        words(title, "Effective", 0.4, BLUE);
                    }
                });
            tally_row(
                tallies,
                atlases,
                TALLY_ICONS
                    .into_iter()
                    .zip([tally.aces, tally.faces, tally.numbers]),
            );
            tally_row(
                tallies,
                atlases,
                (0..2).map(|s| (SUIT_ICONS[s], tally.suits[s])),
            );
            tally_row(
                tallies,
                atlases,
                (2..4).map(|s| (SUIT_ICONS[s], tally.suits[s])),
            );
        });
    });
}

/// The rank column: the rank on an L_BLACK pill, then its count.
fn rank_column(ui: &mut ChildSpawnerCommands, tally: &DeckTally) {
    ui.spawn(column(0.0)).with_children(|ranks| {
        for (rank, count_of) in RANKS.chars().zip(tally.ranks) {
            ranks
                .spawn(Node {
                    padding: UiRect::all(px(0.07 * TILE)),
                    column_gap: px(0.07 * TILE),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|line| {
                    line.spawn((
                        Node {
                            min_width: px(0.5 * TILE),
                            padding: UiRect::all(px(0.04 * TILE)),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        PixelBox::new(L_BLACK).emboss(0.04),
                    ))
                    .with_children(|pill| {
                        let rank = if rank == 'T' {
                            "10".into()
                        } else {
                            rank.to_string()
                        };
                        words(pill, rank, 0.35, JOKER_GREY);
                    });
                    line.spawn(Node {
                        min_width: px(0.4 * TILE),
                        justify_content: JustifyContent::End,
                        ..default()
                    })
                    .with_children(|value| count(value, count_of, 0.45));
                });
        }
    });
}

/// One `title` area per suit holding cards: 6.5 cards wide, 0.6 card tall, cards at 0.7.
fn suit_rows(
    ui: &mut ChildSpawnerCommands,
    atlases: &Atlases,
    cards: &[DeckCardView],
    remaining: bool,
) {
    let card = Vec2::new(0.7 * CARD_WIDTH, 0.7 * CARD_HEIGHT);
    for (suit, name) in SUITS.into_iter().enumerate() {
        let row: Vec<&DeckCardView> = cards.iter().filter(|card| card.suit == suit).collect();
        if row.is_empty() {
            continue;
        }
        let area = CardArea::new(AreaKind::Title, 6.5 * CARD_W, 0.7 * CARD_W, row.len());
        card_row(ui, area, 0.6 * CARD_H, card, row.len(), |entity, index| {
            let shown = row[index];
            crate::components::tooltip::attach_anchored(
                entity,
                crate::components::tooltip::playing_card(
                    shown.rank,
                    name,
                    shown.enhancement.unwrap_or_default(),
                    shown.edition,
                    shown.seal.unwrap_or_default(),
                    shown.debuffed,
                )
                .technical(super::deck_card_effect(shown))
                .technical(format!(
                    "Debuffed: {} · remaining: {} · effective suits: {}",
                    shown.debuffed, shown.remaining, shown.effective_suits
                )),
                crate::components::tooltip::TipAnchor::Left,
            );
            entity.with_children(|art| {
                let face = crate::components::widgets::PlayingFace {
                    rank: shown.rank,
                    suit: name,
                    enhancement: shown.enhancement.unwrap_or_default(),
                    edition: shown.edition,
                    seal: shown.seal.unwrap_or_default(),
                    debuffed: shown.debuffed,
                    greyed: remaining && !shown.remaining,
                    seed: suit * 64 + index,
                };
                if !crate::components::widgets::playing_card_face(art, atlases, &face, card.x) {
                    crate::components::widgets::fallback_art(
                        art,
                        &format!("{} of {name}", shown.rank),
                        card.x,
                    );
                }
            });
        });
    }
}

pub fn inspect_deck(
    ui: &mut ChildSpawnerCommands,
    view: &RunView,
    atlases: &Atlases,
    deck: DeckChoice,
    remaining: bool,
) {
    let remaining = remaining && view.phase == Phase::Playing;
    let tally = DeckTally::count(&view.deck_view, remaining);
    generic_options(ui, Click::Back, |content| {
        row(content, |tabs| {
            if view.phase == Phase::Playing {
                tile_optional(
                    tabs,
                    "Remaining",
                    RED,
                    (2.8, 0.6),
                    0.4,
                    Some(Click::ShowRemaining(true)),
                    remaining,
                );
            }
            tile_optional(
                tabs,
                "Full Deck",
                RED,
                (2.8, 0.6),
                0.4,
                Some(Click::ShowRemaining(false)),
                !remaining,
            );
        });
        content
            .spawn(Node {
                column_gap: px(0.2 * TILE),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|body| {
                body.spawn((
                    Node {
                        min_width: px(1.5 * TILE),
                        min_height: px(2.0 * TILE),
                        padding: UiRect::right(px(0.1 * TILE)),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    PixelBox::new(BLACK).emboss(0.05),
                ))
                .with_children(|left| {
                    deck_box(left, atlases, deck, &tally);
                    rank_column(left, &tally);
                });
                body.spawn((
                    Node {
                        padding: UiRect::all(px(0.1 * TILE)),
                        ..column(0.0)
                    },
                    PixelBox::new(BLACK).emboss(0.05),
                ))
                .with_children(|rows| suit_rows(rows, atlases, &view.deck_view, remaining));
            });
        if tally.modded() {
            content
                .spawn(Node {
                    min_height: px(0.8 * TILE),
                    column_gap: px(0.1 * TILE),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|note| {
                    note.spawn((
                        Node {
                            width: px(0.6 * TILE),
                            height: px(0.6 * TILE),
                            ..default()
                        },
                        // `mix_colours(BLUE, WHITE, 0.7)`: 70% blue.
                        PixelBox::new(BLUE.mix(&Color::WHITE, 0.3)),
                    ));
                    words(
                        note,
                        "Blue counts are the cards as they play: debuffed cards drop out, Wild cards count for every suit",
                        0.3,
                        Color::WHITE,
                    );
                });
        }
    });
}

#[cfg(test)]
mod tests;

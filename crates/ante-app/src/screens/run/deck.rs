use super::*;

/// "View Deck" over the deck while it is hovered.
#[derive(Component)]
pub(crate) struct DeckHoverLabel;

pub(super) fn deck_hover_label(ui: &mut ChildSpawnerCommands) {
    ui.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: px(DECK_AREA.x),
            height: px(DECK_AREA.y),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Visibility::Hidden,
        DeckHoverLabel,
        Pickable::IGNORE,
    ))
    .with_children(|centre| {
        centre
            .spawn((
                Node {
                    padding: UiRect::all(px(0.1 * TILE)),
                    ..column(0.0)
                },
                PixelBox::new(Color::srgba(0.0, 0.0, 0.0, 0.45)),
                Pickable::IGNORE,
            ))
            .with_children(|text| {
                shadowed_label(text, "View", text_px(0.5), INK);
                shadowed_label(text, "Deck", text_px(0.4), INK);
            });
    });
}

pub fn inspect_deck(
    ui: &mut ChildSpawnerCommands,
    view: &RunView,
    atlases: &Atlases,
    deck: DeckChoice,
    remaining: bool,
) {
    let remaining = remaining && view.phase == Phase::Playing;
    let mut ranks = [0; 13];
    let mut suits = [0; 4];
    for card in view
        .deck_view
        .iter()
        .filter(|card| !remaining || card.remaining)
    {
        if let Some(rank) = RANKS.find(card.rank) {
            ranks[rank] += 1;
        }
        suits[card.suit] += 1;
    }
    modal(ui, 1200.0, 16.0, |panel| {
        row(panel, |tabs| {
            if view.phase == Phase::Playing {
                tile_optional(
                    tabs,
                    "Remaining",
                    RED,
                    (2.8, 0.78),
                    0.5,
                    Some(Click::ShowRemaining(true)),
                    remaining,
                );
            }
            tile_optional(
                tabs,
                "Full Deck",
                RED,
                (2.8, 0.78),
                0.5,
                Some(Click::ShowRemaining(false)),
                !remaining,
            );
        });
        panel
            .spawn(Node {
                width: percent(100),
                align_items: AlignItems::Center,
                column_gap: px(12),
                ..default()
            })
            .with_children(|body| {
                body.spawn(box_node(
                    Node {
                        width: px(208),
                        padding: UiRect::all(px(8)),
                        ..column(11.0)
                    },
                    WELL,
                ))
                .with_children(|info| {
                    label(info, deck.name(), 28.0, INK);
                    description(
                        info,
                        match deck {
                            DeckChoice::Red => "+1 discard\neach round",
                            DeckChoice::Blue => "+1 hand\neach round",
                            _ => deck.effect(),
                        },
                        185.0,
                        85.0,
                    );
                    label(
                        info,
                        format!("Base cards  {}", suits.iter().sum::<usize>()),
                        24.0,
                        RED,
                    );
                    for (suit, count) in SUITS.into_iter().zip(suits) {
                        label(info, format!("{suit}  {count}"), 22.0, INK);
                    }
                });
                body.spawn(box_node(
                    Node {
                        width: px(95),
                        padding: UiRect::all(px(8)),
                        ..column(1.0)
                    },
                    WELL,
                ))
                .with_children(|counts| {
                    for (rank, count) in RANKS.chars().zip(ranks) {
                        let rank = if rank == 'T' {
                            "10".to_owned()
                        } else {
                            rank.to_string()
                        };
                        label(counts, format!("{rank}   {count}"), 24.0, INK);
                    }
                });
                body.spawn(Node {
                    width: px(820),
                    ..column(7.0)
                })
                .with_children(|lanes| {
                    for (suit, name) in SUITS.into_iter().enumerate() {
                        let cards = view
                            .deck_view
                            .iter()
                            .filter(|card| card.suit == suit && (!remaining || card.remaining));
                        let count = suits[suit];
                        lanes
                            .spawn(box_node(
                                Node {
                                    width: percent(100),
                                    height: px(101),
                                    ..default()
                                },
                                WELL,
                            ))
                            .with_children(|lane| {
                                if count == 0 {
                                    label(lane, format!("{name} — 0"), 20.0, MUTED);
                                }
                                let width = 64.0;
                                let step = ((820.0 - width - 16.0)
                                    / count.saturating_sub(1).max(1) as f32)
                                    .min(66.0);
                                for (index, card) in cards.enumerate() {
                                    // Atlas cards draw their own rounded face; only the text
                                    // fallback needs a card-coloured box behind it.
                                    let backing = if atlases.card(card.rank, name).is_some() {
                                        Color::NONE
                                    } else {
                                        CREAM
                                    };
                                    let mut entity = lane.spawn(box_node(
                                        Node {
                                            position_type: PositionType::Absolute,
                                            left: px(8.0 + index as f32 * step),
                                            top: px(6),
                                            width: px(width),
                                            height: px(85.6),
                                            align_items: AlignItems::Center,
                                            justify_content: JustifyContent::Center,
                                            ..default()
                                        },
                                        backing,
                                    ));
                                    card_hover(&mut entity);
                                    crate::components::tooltip::attach_anchored(
                                &mut entity,
                                crate::components::tooltip::playing_card(
                                    card.rank,
                                    name,
                                    card.enhancement.unwrap_or_default(),
                                    card.edition,
                                    card.seal.unwrap_or_default(),
                                    card.debuffed,
                                )
                                .technical(deck_card_effect(card))
                                .technical(format!("Deck slot #{index}"))
                                .technical(format!(
                                    "Debuffed: {} · remaining: {} · effective suits: {}",
                                    card.debuffed, card.remaining, card.effective_suits
                                )),
                                crate::components::tooltip::TipAnchor::Left,
                            );
                                    entity.with_children(|art| {
                                        let face = crate::components::widgets::PlayingFace {
                                            rank: card.rank,
                                            suit: name,
                                            enhancement: card.enhancement.unwrap_or_default(),
                                            edition: card.edition,
                                            seal: card.seal.unwrap_or_default(),
                                            debuffed: card.debuffed,
                                            seed: index,
                                        };
                                        if !crate::components::widgets::playing_card_face(
                                            art, atlases, &face, width,
                                        ) {
                                            let rank = if card.rank == 'T' {
                                                "10".to_owned()
                                            } else {
                                                card.rank.to_string()
                                            };
                                            label(
                                                art,
                                                rank,
                                                30.0,
                                                if suit % 2 == 1 { RED } else { WELL },
                                            );
                                        }
                                    });
                                }
                            });
                    }
                });
            });
        tile_button(panel, "Back", ORANGE, (15.0, 0.66), 0.49, Click::Back);
    });
}

pub fn deck_preview(ui: &mut ChildSpawnerCommands, view: &RunView) {
    let mut counts = [[0usize; 13]; 4];
    let mut stones = 0;
    let mut effective = [0usize; 4];
    for card in view.deck_view.iter().filter(|card| card.remaining) {
        if card.enhancement == Some("Stone") {
            stones += 1;
        } else if let Some(rank) = RANKS.find(card.rank) {
            counts[card.suit][rank] += 1;
            for (suit, count) in effective.iter_mut().enumerate() {
                *count += usize::from(card.effective_suits & (1 << suit) != 0);
            }
        }
    }
    ui.spawn((
        box_node(
            Node {
                padding: UiRect::all(px(8)),
                align_items: AlignItems::Center,
                ..column(3.0)
            },
            WELL,
        ),
        Pickable::IGNORE,
    ))
    .with_children(|panel| {
        let totals = std::array::from_fn::<_, 13, _>(|rank| {
            counts.iter().map(|suit| suit[rank]).sum::<usize>()
        });
        let colors = [Color::srgb(0.28, 0.20, 0.48), RED, BLUE, ORANGE];
        for (suit, rank_counts) in std::iter::once((None, totals)).chain(
            counts
                .into_iter()
                .enumerate()
                .map(|(i, row)| (Some(i), row)),
        ) {
            panel
                .spawn((
                    Node {
                        column_gap: px(2),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|row| {
                    let title = suit.map_or_else(
                        || "Rank".into(),
                        |i| {
                            let base = rank_counts.iter().sum::<usize>();
                            if effective[i] == base {
                                format!("{} {base}", SUITS[i])
                            } else {
                                format!("{} {base} ({})", SUITS[i], effective[i])
                            }
                        },
                    );
                    row.spawn((
                        Node {
                            width: px(114),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|cell| label(cell, title, 17.0, INK));
                    for (index, count) in rank_counts.into_iter().enumerate() {
                        row.spawn((
                            box_node(
                                Node {
                                    width: px(33),
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                suit.map_or(PANEL, |i| colors[i]),
                            ),
                            Pickable::IGNORE,
                        ))
                        .with_children(|cell| {
                            let text = if suit.is_none() {
                                let rank = RANKS.as_bytes()[index] as char;
                                format!(
                                    "{}\n{count}",
                                    if rank == 'T' {
                                        "10".into()
                                    } else {
                                        rank.to_string()
                                    }
                                )
                            } else {
                                count.to_string()
                            };
                            label(cell, text, 18.0, if count == 0 { MUTED } else { INK });
                        });
                    }
                });
        }
        if stones > 0 {
            label(panel, format!("Stone: {stones}"), 17.0, INK);
        }
    });
}

/// Deck view lines: only the attributes the card actually has, like the original.
pub(super) fn deck_card_effect(card: &ante_core::contract::DeckCardView) -> String {
    let mut lines = Vec::new();
    if let Some(enhancement) = card.enhancement {
        lines.push(format!("Enhancement: {enhancement}"));
    }
    if !matches!(card.edition, "" | "Base" | "None") {
        lines.push(format!("Edition: {}", card.edition));
    }
    if let Some(seal) = card.seal {
        lines.push(format!("Seal: {seal}"));
    }
    if card.debuffed {
        lines.push("Debuffed".to_owned());
    }
    lines.join("\n")
}

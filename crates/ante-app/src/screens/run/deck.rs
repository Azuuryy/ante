use super::*;

/// "View Deck" over the deck while it is hovered.
#[derive(Component)]
pub(crate) struct DeckHoverLabel;

/// The `view_deck` UIBox of `CardArea:draw`: "View" at scale 0.48 and "Deck" at 0.38 in
/// white, on black at 50% with 0.05 padding, centred on the top card of the pile
/// (`major = self.cards[1]`); `RunBoard` moves it as the pile shrinks.
pub(super) fn deck_hover_label(ui: &mut ChildSpawnerCommands) -> Entity {
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
        UiTransform::IDENTITY,
        Pickable::IGNORE,
    ))
    .with_children(|centre| {
        centre
            .spawn((
                Node {
                    padding: UiRect::all(px(0.05 * TILE)),
                    max_width: px(2.0 * TILE),
                    ..column(0.0)
                },
                PixelBox::new(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                Pickable::IGNORE,
            ))
            .with_children(|text| {
                shadowed_label(text, "View", text_px(0.48), Color::WHITE);
                shadowed_label(text, "Deck", text_px(0.38), Color::WHITE);
            });
    })
    .id()
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

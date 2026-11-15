use super::*;
use crate::components::tiles::{cycle_arrow, deck_preview_large};

/// `G.C.UI.TEXT_DARK`.
const TEXT_DARK: Color = Color::srgb(
    0x4F as f32 / 255.0,
    0x63 as f32 / 255.0,
    0x67 as f32 / 255.0,
);

/// `create_UIBox_your_collection_decks`: an option cycle whose middle is a black box with
/// the deck pile and an L_BLACK column holding the deck name and its description on
/// white, with one pip per deck under it.
pub(super) fn decks(
    ui: &mut ChildSpawnerCommands,
    session: &Session,
    profile: &Profile,
    atlases: &Atlases,
) {
    let deck = session.navigation.deck.unwrap_or(DeckChoice::Red);
    let condition = deck.unlock();
    let locked = !profile.is_unlocked(deck.id(), condition);
    generic_options(ui, Click::Back, |content| {
        content
            .spawn(Node {
                padding: UiRect::all(px(0.1 * TILE)),
                column_gap: px(0.1 * TILE),
                align_items: AlignItems::Stretch,
                ..default()
            })
            .with_children(|cycle| {
                cycle_arrow(cycle, "<", Some(Click::ShiftDeck(-1)));
                cycle
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: px(0.05 * TILE),
                        ..default()
                    })
                    .with_children(|middle| {
                        deck_box(middle, atlases, deck, locked, condition);
                        pips(middle, deck);
                    });
                cycle_arrow(cycle, ">", Some(Click::ShiftDeck(1)));
            });
    });
}

fn deck_box(
    ui: &mut ChildSpawnerCommands,
    atlases: &Atlases,
    deck: DeckChoice,
    locked: bool,
    condition: Option<&'static str>,
) {
    ui.spawn((
        Node {
            min_width: px(2.5 * TILE),
            padding: UiRect::all(px(0.1 * TILE)),
            ..default()
        },
        PixelBox::new(BLACK).emboss(0.05),
    ))
    .with_children(|outer| {
        outer
            .spawn(Node {
                padding: UiRect::all(px(0.2 * TILE)),
                column_gap: px(0.2 * TILE),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|row| {
                let mut pile = row.spawn(Node::default());
                tooltip::attach(&mut pile, deck_tip(deck, locked, condition));
                pile.with_children(|art| {
                    deck_preview_large(art, deck, atlases.deck_back(deck));
                    if locked {
                        lock_overlay(art);
                    }
                });
                row.spawn((
                    Node {
                        min_width: px(3.7 * TILE),
                        min_height: px(2.1 * TILE),
                        padding: UiRect::all(px(0.1 * TILE)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: px(0.1 * TILE),
                        ..default()
                    },
                    PixelBox::new(L_BLACK),
                ))
                .with_children(|info| {
                    info.spawn((
                        Node {
                            min_width: px(4.0 * TILE),
                            max_width: px(4.0 * TILE),
                            min_height: px(0.6 * TILE),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        PixelBox::new(L_BLACK).emboss(0.1),
                    ))
                    .with_children(|name| words(name, deck.name(), 0.5, INK));
                    info.spawn((
                        Node {
                            width: percent(100),
                            min_height: px(2.2 * TILE),
                            padding: UiRect::all(px(0.1 * TILE)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        PixelBox::new(Color::WHITE).emboss(0.1),
                    ))
                    .with_children(|text| {
                        let description = if locked {
                            format!("Locked\n{}", condition.unwrap_or("Keep playing to unlock"))
                        } else {
                            deck.effect().to_owned()
                        };
                        text.spawn((
                            Node {
                                max_width: px(3.8 * TILE),
                                ..default()
                            },
                            Text::new(description),
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
            });
    });
}

/// `choice_pips`: a 0.1 square per deck, white for the shown one.
fn pips(ui: &mut ChildSpawnerCommands, deck: DeckChoice) {
    ui.spawn(Node {
        padding: UiRect::all(px(0.02 * TILE)),
        column_gap: px(0.04 * TILE),
        ..default()
    })
    .with_children(|row| {
        for choice in DeckChoice::ALL {
            row.spawn((
                Node {
                    width: px(0.1 * TILE),
                    height: px(0.1 * TILE),
                    ..default()
                },
                PixelBox::new(if choice == deck { Color::WHITE } else { BLACK }),
            ));
        }
    });
}

use super::*;
use crate::components::tooltip::ink::{PLANET, SPECTRAL, TAROT};
use menu_ui::RED;

/// `UIBox_button`: a coloured button `minw` × `minh` with its label at `scale` and, when
/// given, the "found / total" count under it at 0.35.
fn collection_button(
    ui: &mut ChildSpawnerCommands,
    label: &str,
    count: Option<(usize, usize)>,
    (minw, minh, scale): (f32, f32, f32),
    color: Color,
    action: Click,
) {
    let mut button = ui.spawn((
        Node {
            min_width: px(minw * TILE),
            min_height: px(minh * TILE),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        PixelBox::new(color).shadow(),
    ));
    menu_ui::attach_button(&mut button, color, action);
    button.with_children(|text| {
        words(text, label, scale, INK);
        if let Some((found, total)) = count {
            text.spawn(Node {
                min_height: px(0.4 * TILE),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|tally| {
                words(
                    tally,
                    format!("{found} / {total}"),
                    0.35,
                    Color::srgba(1.0, 1.0, 1.0, 0.9),
                );
            });
        }
    });
}

fn column(ui: &mut ChildSpawnerCommands, content: impl FnOnce(&mut ChildSpawnerCommands)) {
    ui.spawn(Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        padding: UiRect::all(px(0.15 * TILE)),
        row_gap: px(0.15 * TILE),
        ..default()
    })
    .with_children(content);
}

/// `create_UIBox_your_collection`: Jokers, Decks, Vouchers and the consumables group on the
/// left; enhancements, seals, editions, boosters, tags and blinds on the right.
pub(super) fn hub(ui: &mut ChildSpawnerCommands, profile: &Profile, backend: &dyn GameBackend) {
    let tally = |group: ContentGroup| {
        let items = backend.catalog(group);
        let found = items
            .iter()
            .filter(|item| profile.discovered(item.id.0))
            .count();
        Some((found, items.len()))
    };
    let decks = {
        let unlocked = DeckChoice::ALL
            .iter()
            .filter(|deck| profile.is_unlocked(deck.id(), deck.unlock()))
            .count();
        Some((unlocked, DeckChoice::ALL.len()))
    };
    let items = |group| Click::CollectionItems(CollectionView::Items(group));
    generic_options(ui, Click::Back, |content| {
        content.spawn(Node::default()).with_children(|columns| {
            column(columns, |left| {
                collection_button(
                    left,
                    "Jokers",
                    tally(ContentGroup::Jokers),
                    (5.0, 1.7, 0.6),
                    RED,
                    Click::CollectionJokers,
                );
                collection_button(
                    left,
                    "Decks",
                    decks,
                    (5.0, 0.9, 0.5),
                    RED,
                    Click::CollectionDecks,
                );
                collection_button(
                    left,
                    "Vouchers",
                    tally(ContentGroup::Vouchers),
                    (5.0, 0.9, 0.5),
                    RED,
                    items(ContentGroup::Vouchers),
                );
                left.spawn((
                    Node {
                        padding: UiRect::all(px(0.1 * TILE)),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    PixelBox::new(BLACK),
                ))
                .with_children(|group| {
                    // The vertical "CONSUMABLES" label (`vert = true`).
                    group
                        .spawn(Node {
                            width: px(0.45 * TILE),
                            align_self: AlignSelf::Stretch,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|side| {
                            side.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    ..default()
                                },
                                Text::new("CONSUMABLES"),
                                TextFont {
                                    font_size: bevy::text::FontSize::Px(text_px(0.45)),
                                    ..default()
                                },
                                TextColor(L_BLACK),
                                TextLayout::no_wrap(),
                                UiTransform::from_rotation(Rot2::degrees(-90.0)),
                                Pickable::IGNORE,
                            ));
                        });
                    column(group, |sets| {
                        for (name, set, color) in [
                            ("Tarot Cards", ContentGroup::Tarots, TAROT),
                            ("Planet Cards", ContentGroup::Planets, PLANET),
                            ("Spectral Cards", ContentGroup::Spectrals, SPECTRAL),
                        ] {
                            collection_button(
                                sets,
                                name,
                                tally(set),
                                (4.0, 0.9, 0.5),
                                color,
                                items(set),
                            );
                        }
                    });
                });
            });
            column(columns, |right| {
                for (name, group, counted) in [
                    ("Enhanced Cards", ContentGroup::Enhancements, false),
                    ("Seals", ContentGroup::Seals, false),
                    ("Editions", ContentGroup::Editions, true),
                    ("Booster Packs", ContentGroup::Boosters, true),
                    ("Tags", ContentGroup::Tags, true),
                ] {
                    collection_button(
                        right,
                        name,
                        counted.then(|| tally(group)).flatten(),
                        (5.0, 0.9, 0.5),
                        RED,
                        items(group),
                    );
                }
                collection_button(
                    right,
                    "Blinds",
                    tally(ContentGroup::Blinds),
                    (5.0, 2.0, 0.5),
                    RED,
                    items(ContentGroup::Blinds),
                );
            });
        });
    });
}

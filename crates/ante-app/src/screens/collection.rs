use crate::{
    app::{
        navigation::DeckChoice,
        session::{Click, CollectionView, Session},
    },
    components::{
        theme::{DISABLED, INK, MUTED},
        tiles as menu_ui, tooltip,
        widgets::{atlas_image, control, fallback_art, label, row, wrap_row},
    },
    services::{assets::catalog::Atlases, profile::Profile},
};
use ante_core::contract::{Content, ContentGroup, GameBackend};
use bevy::{picking::prelude::Pickable, prelude::*};

mod art;

use art::*;

pub(crate) fn show_collection(
    ui: &mut ChildSpawnerCommands,
    session: &Session,
    profile: &Profile,
    atlases: &Atlases,
    backend: &dyn GameBackend,
) {
    use menu_ui::{BLUE, ORANGE, PALE_GREEN, RED, tile_button as tile};
    menu_ui::screen_heading(ui, "COLLECTION");
    // Hub tiles show discovered / total, like the original collection buttons.
    let count_label = |name: &str, group: ContentGroup| {
        let items = backend.catalog(group);
        let found = items
            .iter()
            .filter(|item| profile.discovered(item.id.0))
            .count();
        format!("{name}  {found}/{}", items.len())
    };
    menu_ui::panel(ui, |panel| {
        match session.collection_view {
            CollectionView::Hub => {
                // Two columns side by side; they never wrap under each other.
                panel
                    .spawn(Node {
                        column_gap: menu_ui::tiles(0.15),
                        ..default()
                    })
                    .with_children(|columns| {
                        columns
                            .spawn(Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: menu_ui::tiles(0.12),
                                ..default()
                            })
                            .with_children(|column| {
                                tile(
                                    column,
                                    &count_label("JOKERS", ContentGroup::Jokers),
                                    RED,
                                    (4.7, 1.1),
                                    0.5,
                                    Click::CollectionJokers,
                                );
                                tile(
                                    column,
                                    "DECKS",
                                    BLUE,
                                    (4.7, 0.75),
                                    0.42,
                                    Click::CollectionDecks,
                                );
                                for (name, group) in [
                                    ("VOUCHERS", ContentGroup::Vouchers),
                                    ("TAROT", ContentGroup::Tarots),
                                    ("PLANET", ContentGroup::Planets),
                                    ("SPECTRAL", ContentGroup::Spectrals),
                                ] {
                                    tile(
                                        column,
                                        &count_label(name, group),
                                        RED,
                                        (4.7, 0.65),
                                        0.33,
                                        Click::CollectionItems(CollectionView::Items(group)),
                                    );
                                }
                            });
                        columns
                            .spawn(Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: menu_ui::tiles(0.12),
                                ..default()
                            })
                            .with_children(|column| {
                                for (name, group) in [
                                    ("ENHANCED CARDS", ContentGroup::Enhancements),
                                    ("SEALS", ContentGroup::Seals),
                                    ("EDITIONS", ContentGroup::Editions),
                                    ("BOOSTERS", ContentGroup::Boosters),
                                    ("TAGS", ContentGroup::Tags),
                                    ("BLINDS", ContentGroup::Blinds),
                                ] {
                                    tile(
                                        column,
                                        &count_label(name, group),
                                        PALE_GREEN,
                                        (4.7, 0.72),
                                        0.33,
                                        Click::CollectionItems(CollectionView::Items(group)),
                                    );
                                }
                            });
                    });
            }
            CollectionView::Decks => {
                let deck = session.navigation.deck.unwrap_or(DeckChoice::Red);
                let condition = deck.unlock();
                let locked = !profile.is_unlocked(deck.id(), condition);
                row(panel, |selector| {
                    tile(selector, "<", RED, (0.8, 2.3), 0.6, Click::ShiftDeck(-1));
                    let mut preview = selector.spawn(Node::default());
                    tooltip::attach(&mut preview, deck_tip(deck, locked, condition));
                    preview.with_children(|art| {
                        menu_ui::deck_preview_large(art, deck, atlases.deck_back(deck));
                        if locked {
                            lock_overlay(art);
                        }
                    });
                    tile(selector, ">", RED, (0.8, 2.3), 0.6, Click::ShiftDeck(1));
                });
                label(panel, deck.name(), menu_ui::text_px(0.5), INK);
                label(panel, deck.effect(), menu_ui::text_px(0.38), MUTED);
                if locked {
                    label(
                        panel,
                        format!("Locked — {}", condition.unwrap_or("Keep playing to unlock")),
                        menu_ui::text_px(0.34),
                        MUTED,
                    );
                }
            }
            CollectionView::Items(group) => {
                const PAGE_SIZE: usize = 15;
                let catalog = backend.catalog(group);
                let pages = catalog.len().div_ceil(PAGE_SIZE).max(1);
                let page = session.navigation.collection_page.min(pages - 1);
                if !profile.is_writable() {
                    label(
                        panel,
                        "Profile unreadable: discoveries kept for this session",
                        17.0,
                        MUTED,
                    );
                }
                for page_items in
                    catalog[page * PAGE_SIZE..((page + 1) * PAGE_SIZE).min(catalog.len())].chunks(5)
                {
                    wrap_row(panel, |entries| {
                        for item in page_items {
                            let id = item.id.0;
                            let (name, supported) = (item.name.as_str(), item.supported);
                            let unlocked = profile.is_unlocked(id, item.unlock);
                            let discovered = profile.discovered(id);
                            let locked = !unlocked;
                            let sprite = collection_sprite(atlases, group, id);
                            control(
                                entries,
                                DISABLED,
                                None,
                                (125.0, 135.0),
                                Some(collection_tip(item, group, discovered, locked)),
                                |entry| {
                                    entry
                                        .spawn(Node {
                                            width: px(66),
                                            height: px(88),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        })
                                        .with_children(|art| {
                                            if (discovered || locked)
                                                && let Some(sprite) = sprite.clone()
                                            {
                                                collection_art(art, atlases, group, id, sprite);
                                            } else {
                                                fallback_art(
                                                    art,
                                                    if locked || !discovered { "?" } else { name },
                                                    66.0,
                                                );
                                            }
                                            if locked {
                                                lock_overlay(art);
                                            }
                                        });
                                    let state = if locked {
                                        "Locked"
                                    } else if !supported {
                                        "Unavailable"
                                    } else if discovered {
                                        "Discovered"
                                    } else {
                                        "Undiscovered"
                                    };
                                    label(entry, state, 12.0, MUTED);
                                },
                            );
                        }
                    });
                }
                row(panel, |controls| {
                    menu_ui::tile_optional(
                        controls,
                        "<",
                        RED,
                        (0.8, 0.65),
                        0.4,
                        page.checked_sub(1).map(Click::Collection),
                        false,
                    );
                    label(
                        controls,
                        format!("PAGE {} / {}", page + 1, pages),
                        menu_ui::text_px(0.35),
                        INK,
                    );
                    menu_ui::tile_optional(
                        controls,
                        ">",
                        RED,
                        (0.8, 0.65),
                        0.4,
                        (page + 1 < pages).then_some(Click::Collection(page + 1)),
                        false,
                    );
                });
            }
        }
        tile(panel, "BACK", ORANGE, (2.8, 0.75), 0.42, Click::Back);
    });
}

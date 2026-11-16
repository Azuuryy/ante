//! The Collection, after `create_UIBox_your_collection*`: the hub of buttons, the card
//! pages (one dark box of card areas with a page cycle), the Tags and Blinds grids and the
//! deck carousel, each inside `create_UIBox_generic_options`.

use crate::{
    app::{
        navigation::DeckChoice,
        session::{Click, CollectionView, Session},
    },
    components::{
        pixel_rect::PixelBox,
        theme::{INK, MUTED},
        tiles::{self as menu_ui, BLACK, CARD_H, CARD_W, L_BLACK, TILE, generic_options, text_px},
        tooltip,
        widgets::{atlas_image, fallback_art, label, shadowed_label},
    },
    services::{assets::catalog::Atlases, profile::Profile},
};
use ante_core::contract::{Content, ContentGroup, GameBackend};
use bevy::{picking::prelude::Pickable, prelude::*};

mod art;
mod cards;
mod decks;
mod grids;
mod hub;
mod layout;

use art::*;

pub(crate) fn show_collection(
    ui: &mut ChildSpawnerCommands,
    session: &Session,
    profile: &Profile,
    atlases: &Atlases,
    backend: &dyn GameBackend,
) {
    match session.collection_view {
        CollectionView::Hub => hub::hub(ui, profile, backend),
        CollectionView::Decks => decks::decks(ui, session, profile, atlases),
        CollectionView::Items(ContentGroup::Tags) => {
            grids::tags(ui, profile, atlases, backend.catalog(ContentGroup::Tags));
        }
        CollectionView::Items(ContentGroup::Blinds) => {
            grids::blinds(ui, profile, atlases, backend.catalog(ContentGroup::Blinds));
        }
        CollectionView::Items(group) => {
            let mut catalog = Vec::new();
            if group == ContentGroup::Editions {
                // The original page starts with `e_base`, the plain Joker.
                catalog.push(Content {
                    id: ante_core::contract::ContentId(BASE_EDITION),
                    name: "Base".to_owned(),
                    description: "No edition".to_owned(),
                    cost: 0,
                    supported: true,
                    unlock: None,
                    rarity: None,
                });
            }
            catalog.extend(backend.catalog(group).iter().cloned());
            cards::card_page(
                ui,
                session.navigation.collection_page,
                profile,
                atlases,
                group,
                &catalog,
            );
        }
    }
}

/// A text at an original scale with the usual shadow.
fn words(ui: &mut ChildSpawnerCommands, text: impl Into<String>, scale: f32, color: Color) {
    shadowed_label(ui, text, text_px(scale), color);
}

/// The note shown when the profile cannot be saved.
fn unsaved_note(ui: &mut ChildSpawnerCommands, profile: &Profile) {
    if !profile.is_writable() {
        label(
            ui,
            "Profile unreadable: discoveries kept for this session",
            text_px(0.3),
            MUTED,
        );
    }
}

/// The plain edition the Editions page starts with.
const BASE_EDITION: &str = "e_base";

/// Whether the entry shows as discovered (enhancements, seals and the base edition always
/// do) and whether it is locked.
fn shown(profile: &Profile, group: ContentGroup, item: &Content) -> (bool, bool) {
    let locked = !profile.is_unlocked(item.id.0, item.unlock);
    let discovered =
        profile.discovered(item.id.0) || !hides_undiscovered(group) || item.id.0 == BASE_EDITION;
    (discovered, locked)
}

#[cfg(test)]
mod tests;

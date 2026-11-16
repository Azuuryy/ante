//! Highlighted shop, pack and inventory cards, applied in place.
//!
//! The original highlights a card (`Card:highlight`) without rebuilding its area: the card
//! rises, comes to the front and shows its buttons. Cards are spawned unselected with their
//! buttons hidden, and [`apply_selection`] follows the session's selection, so selecting a
//! card no longer rebuilds the shop, the pack or the inventories.

use ante_core::contract::Command;
use bevy::prelude::*;

use crate::{
    app::session::Session,
    components::{
        card_drag::{CardDrag, Lane},
        widgets::HoverMotion,
    },
};

/// What selects a card: a shop or pack action, or an inventory slot of a revision.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum SelectKey {
    Offer(Command),
    Inventory(Lane, usize, u64),
}

/// How a highlighted card is layered: inside its area (`ZIndex`) or across the screen
/// (`GlobalZIndex`, used by the inventories so a dragged card stays on top).
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Layer {
    Local,
    Global,
}

/// A card that can be highlighted.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct Selectable {
    pub(crate) key: SelectKey,
    pub(crate) layer: Layer,
    /// Layer when not highlighted, and when highlighted.
    pub(crate) rest_z: i32,
    pub(crate) chosen_z: i32,
    /// Lift (px) of the highlighted card.
    pub(crate) lift: f32,
}

/// A child of a [`Selectable`] shown only while the card is highlighted (its Buy, Use or
/// Sell buttons).
#[derive(Component, Default)]
pub(crate) struct SelectionButtons;

/// Whether `key` is the session's current selection.
pub(crate) fn is_selected(session: &Session, key: SelectKey) -> bool {
    match key {
        SelectKey::Offer(action) => session.shop_selection == Some(action),
        SelectKey::Inventory(lane, index, revision) => {
            session.playback.is_none()
                && session.inventory_selection == Some((lane, index, revision))
        }
    }
}

type SelectableParts = (
    Entity,
    &'static Selectable,
    Option<&'static Children>,
    Option<&'static mut ZIndex>,
    Option<&'static mut GlobalZIndex>,
    Option<&'static mut HoverMotion>,
);

/// Applies the session's selection to every selectable card, when it changes or when cards
/// are spawned. A card held by a drag keeps the drag's layer.
pub(crate) fn apply_selection(
    session: Res<Session>,
    drag: Res<CardDrag>,
    added: Query<(), Added<Selectable>>,
    mut cards: Query<SelectableParts>,
    mut buttons: Query<&mut Node, With<SelectionButtons>>,
) {
    if !session.is_changed() && added.is_empty() {
        return;
    }
    for (entity, card, children, local, global, motion) in &mut cards {
        let chosen = is_selected(&session, card.key);
        let z = if chosen { card.chosen_z } else { card.rest_z };
        match card.layer {
            Layer::Local => {
                if let Some(mut layer) = local {
                    layer.set_if_neq(ZIndex(z));
                }
            }
            Layer::Global => {
                if let Some(mut layer) = global
                    && !drag.holds(entity)
                {
                    layer.set_if_neq(GlobalZIndex(z));
                }
            }
        }
        if let Some(mut motion) = motion {
            let raise = if chosen { card.lift } else { 0.0 };
            if motion.raise != raise {
                motion.raise = raise;
            }
        }
        let display = if chosen { Display::Flex } else { Display::None };
        for &child in children.into_iter().flatten() {
            if let Ok(mut node) = buttons.get_mut(child)
                && node.display != display
            {
                node.display = display;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::session::Session;
    use ante_core::contract::CampaignGoal;

    fn world() -> (App, Entity, Entity) {
        let mut app = App::new();
        let session = Session::new(
            CampaignGoal::Standard,
            crate::services::assets::import::AssetSettings::load(),
        );
        app.insert_resource(session)
            .init_resource::<CardDrag>()
            .add_systems(Update, apply_selection);
        let buttons = app
            .world_mut()
            .spawn((Node::default(), SelectionButtons))
            .id();
        let card = app
            .world_mut()
            .spawn((
                Selectable {
                    key: SelectKey::Offer(Command::Reroll),
                    layer: Layer::Local,
                    rest_z: 0,
                    chosen_z: 10,
                    lift: 12.0,
                },
                ZIndex(0),
                HoverMotion::default(),
            ))
            .add_child(buttons)
            .id();
        (app, card, buttons)
    }

    #[test]
    fn selecting_a_card_raises_it_and_shows_its_buttons_in_place() {
        let (mut app, card, buttons) = world();
        app.update();
        assert_eq!(
            app.world().get::<Node>(buttons).unwrap().display,
            Display::None
        );
        app.world_mut().resource_mut::<Session>().shop_selection = Some(Command::Reroll);
        app.update();
        let world = app.world();
        assert_eq!(world.get::<ZIndex>(card), Some(&ZIndex(10)));
        assert_eq!(world.get::<HoverMotion>(card).unwrap().raise, 12.0);
        assert_eq!(world.get::<Node>(buttons).unwrap().display, Display::Flex);
        app.world_mut().resource_mut::<Session>().shop_selection = None;
        app.update();
        let world = app.world();
        assert_eq!(world.get::<ZIndex>(card), Some(&ZIndex(0)));
        assert_eq!(world.get::<HoverMotion>(card).unwrap().raise, 0.0);
        assert_eq!(world.get::<Node>(buttons).unwrap().display, Display::None);
    }
}

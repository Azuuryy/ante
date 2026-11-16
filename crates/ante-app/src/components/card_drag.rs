use crate::app::{
    navigation::Screen,
    session::{Click, Session},
};
use crate::components::widgets::HoverMotion;
use ante_core::contract::{CardKey, Command, MoveDirection};
use bevy::{
    ecs::system::EntityCommands,
    picking::prelude::{PointerButton, PointerOver, PointerPress},
    prelude::*,
    window::PrimaryWindow,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Lane {
    Hand,
    Jokers,
    Consumables,
}

#[derive(Component, Clone, Copy)]
pub(crate) struct Slot {
    lane: Lane,
    index: usize,
    click: Option<Click>,
    hand_card: Option<CardKey>,
    selected: bool,
}

struct Grab {
    entity: Entity,
    start: Vec2,
    center: Vec2,
    transform: UiTransform,
    z: i32,
    revision: u64,
    run_id: u64,
    moved: bool,
    origin: usize,
    target: usize,
    /// The dragged card's row when it started moving.
    homes: Vec<Home>,
    /// UI pixels per physical pixel in the row, room scale included.
    px_per_physical: f32,
}

/// A card of the dragged row, as laid out when the drag started moving.
struct Home {
    entity: Entity,
    index: usize,
    /// Centre of its slot, in physical pixels.
    center: f32,
    /// Resting transform, restored on cards without hover easing.
    transform: UiTransform,
}

#[derive(Default)]
struct Sweep {
    touched: Vec<CardKey>,
}

#[derive(Resource, Default)]
pub(crate) struct CardDrag {
    grab: Option<Grab>,
    sweep: Option<Sweep>,
}

impl CardDrag {
    pub(crate) fn active(&self) -> bool {
        self.grab.is_some() || self.sweep.is_some()
    }

    pub(crate) fn holds(&self, entity: Entity) -> bool {
        self.grab.as_ref().is_some_and(|grab| grab.entity == entity)
    }

    pub(crate) fn moving(&self) -> bool {
        self.grab.as_ref().is_some_and(|grab| grab.moved)
    }
}

pub(crate) fn attach(
    card: &mut EntityCommands,
    lane: Lane,
    index: usize,
    click: Option<Click>,
    hand_card: Option<CardKey>,
    selected: bool,
    z: i32,
) {
    card.insert((
        Slot {
            lane,
            index,
            click,
            hand_card,
            selected,
        },
        GlobalZIndex(z),
    ))
    .observe(
        |event: On<PointerPress>,
         mut drag: ResMut<CardDrag>,
         session: Res<Session>,
         windows: Query<&Window, With<PrimaryWindow>>,
         cards: Query<(&Slot, &UiTransform, &UiGlobalTransform, &GlobalZIndex)>| {
            if event.button != PointerButton::Primary
                || drag.active()
                || session.navigation.screen != Screen::Run
                || session.playback.is_some()
            {
                return;
            }
            let (Some(view), Ok(window), Ok((slot, local, global, z))) = (
                &session.confirmed,
                windows.single(),
                cards.get(event.entity),
            ) else {
                return;
            };
            if let Some(start) = window.physical_cursor_position() {
                drag.grab = Some(Grab {
                    entity: event.entity,
                    start,
                    center: global.translation,
                    transform: *local,
                    z: z.0,
                    revision: view.revision,
                    run_id: session.run_id,
                    moved: false,
                    origin: slot.index,
                    target: slot.index,
                    homes: Vec::new(),
                    px_per_physical: 1.0,
                });
            }
        },
    )
    .observe(
        |event: On<PointerOver>,
         mut drag: ResMut<CardDrag>,
         slots: Query<&Slot>,
         mut clicks: MessageWriter<Click>| {
            let Some(sweep) = drag.sweep.as_mut() else {
                return;
            };
            if let Ok(slot) = slots.get(event.entity)
                && slot.lane == Lane::Hand
                && !slot.selected
                && let Some(id) = slot.hand_card
                && !sweep.touched.contains(&id)
            {
                sweep.touched.push(id);
                clicks.write(Click::Play(Command::ToggleCard(id)));
            }
        },
    );
}

type RowCard<'a> = (
    Entity,
    &'a mut Slot,
    &'a ComputedNode,
    &'a UiGlobalTransform,
    &'a mut UiTransform,
    &'a mut GlobalZIndex,
    Option<&'a mut HoverMotion>,
);

pub(crate) fn update(
    mut drag: ResMut<CardDrag>,
    session: Res<Session>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut slots: Query<RowCard>,
    mut clicks: MessageWriter<Click>,
) {
    if mouse.just_pressed(MouseButton::Left) && drag.grab.is_none() && drag.sweep.is_none() {
        drag.sweep = Some(Sweep::default());
    }
    if drag.sweep.is_some() {
        if mouse.pressed(MouseButton::Left) {
            return;
        }
        drag.sweep = None;
        return;
    }
    let Some(grab) = drag.grab.as_mut() else {
        return;
    };
    let Ok((_, slot, node, global, transform, ..)) = slots.get(grab.entity) else {
        drag.grab = None;
        return;
    };
    let (slot, inverse_scale, global, resting_scale) = (
        *slot,
        node.inverse_scale_factor(),
        *global,
        transform.scale.x,
    );
    let window = windows.single().ok().filter(|window| window.focused);
    let cursor = window.and_then(Window::physical_cursor_position);
    let valid = session.navigation.screen == Screen::Run
        && session.playback.is_none()
        && !session.faulted
        && session.run_id == grab.run_id
        && session
            .confirmed
            .as_ref()
            .is_some_and(|view| view.revision == grab.revision);
    if let (true, Some(cursor), Some(window)) = (valid, cursor, window) {
        let delta = cursor - grab.start;
        grab.moved |= delta.length() >= 4.0 * window.resolution.base_scale_factor();
        if mouse.pressed(MouseButton::Left) {
            if grab.moved {
                if grab.homes.is_empty() {
                    grab.homes = row_homes(&slots, slot.lane);
                    grab.px_per_physical =
                        inverse_scale * resting_scale / global.matrix2.x_axis.length().max(1e-6);
                }
                follow_cursor(grab, &mut slots, global, inverse_scale, delta);
                let held_x = grab.center.x + delta.x;
                reorder_row(grab, &mut slots, held_x);
            }
            return;
        }
        if !grab.moved {
            if let Some(click) = slot.click {
                clicks.write(click);
            }
        } else if grab.target != grab.origin {
            send_reorder(grab, slot, &mut clicks);
        }
    } else if valid && window.is_some() && mouse.pressed(MouseButton::Left) {
        return;
    }
    // Back in place, or the confirmed view rebuilds the row in its new order.
    if grab.target == grab.origin || !valid {
        settle_row(grab, &mut slots);
    }
    if let Ok((.., mut transform, mut z, _)) = slots.get_mut(grab.entity) {
        *transform = if grab.z > 0 {
            grab.transform
        } else {
            UiTransform::IDENTITY
        };
        z.set_if_neq(GlobalZIndex(grab.z));
    }
    drag.grab = None;
}

fn row_homes(slots: &Query<RowCard>, lane: Lane) -> Vec<Home> {
    let mut homes: Vec<Home> = slots
        .iter()
        .filter(|(_, slot, ..)| slot.lane == lane)
        .map(|(entity, slot, _, global, transform, ..)| Home {
            entity,
            index: slot.index,
            center: global.translation.x,
            transform: *transform,
        })
        .collect();
    homes.sort_by_key(|home| home.index);
    homes
}

/// The held card follows the pointer, in its own (possibly tilted) frame.
fn follow_cursor(
    grab: &Grab,
    slots: &mut Query<RowCard>,
    global: UiGlobalTransform,
    inverse_scale: f32,
    delta: Vec2,
) {
    let (Some(inverse), Ok((.., mut transform, _, _))) =
        (global.try_inverse(), slots.get_mut(grab.entity))
    else {
        return;
    };
    let shift = inverse.transform_vector2(delta) * grab.transform.scale * inverse_scale;
    transform.translation.x = px(px_value(grab.transform.translation.x) + shift.x);
    transform.translation.y = px(px_value(grab.transform.translation.y) + shift.y);
}

/// Live reorder, like `align_cards` sorting a row by x every frame: the held card takes
/// the slot of every card whose resting centre it passed, and those cards slide over.
fn reorder_row(grab: &mut Grab, slots: &mut Query<RowCard>, held_x: f32) {
    let target = held_target(&grab.homes, grab.origin, held_x);
    grab.target = target;
    for home in &grab.homes {
        let held = home.entity == grab.entity;
        let index = if held {
            target
        } else {
            shifted_index(home.index, grab.origin, target)
        };
        let Ok((_, mut slot, .., mut transform, mut z, motion)) = slots.get_mut(home.entity) else {
            continue;
        };
        slot.index = index;
        // A card keeps the layer of its slot; a selected inventory card stays in front.
        let layer = if held && grab.z != grab.origin as i32 {
            grab.z
        } else {
            index as i32
        };
        z.set_if_neq(GlobalZIndex(layer));
        if held {
            continue;
        }
        let Some(slot) = grab.homes.iter().find(|other| other.index == index) else {
            continue;
        };
        let shift = (slot.center - home.center) * grab.px_per_physical;
        match motion {
            Some(mut motion) => motion.shift_target = shift,
            None => {
                transform.translation.x = px(px_value(home.transform.translation.x) + shift);
            }
        }
    }
}

/// Slot of the held card: one past every other card whose resting centre lies left of it.
fn held_target(homes: &[Home], origin: usize, held_x: f32) -> usize {
    homes
        .iter()
        .filter(|home| home.index != origin && home.center < held_x)
        .count()
}

/// Index of the card resting at `index` once the held card moved from `origin` to `target`.
fn shifted_index(index: usize, origin: usize, target: usize) -> usize {
    if origin < target && index > origin && index <= target {
        index - 1
    } else if target < origin && index >= target && index < origin {
        index + 1
    } else {
        index
    }
}

/// Every other card of the row goes back to its resting slot and layer.
fn settle_row(grab: &Grab, slots: &mut Query<RowCard>) {
    for home in &grab.homes {
        if home.entity == grab.entity {
            continue;
        }
        if let Ok((_, mut slot, .., mut transform, mut z, motion)) = slots.get_mut(home.entity) {
            slot.index = home.index;
            z.set_if_neq(GlobalZIndex(home.index as i32));
            match motion {
                Some(mut motion) => motion.shift_target = 0.0,
                None => *transform = home.transform,
            }
        }
    }
    if let Ok((_, mut slot, ..)) = slots.get_mut(grab.entity) {
        slot.index = grab.origin;
    }
}

fn send_reorder(grab: &Grab, slot: Slot, clicks: &mut MessageWriter<Click>) {
    let to = grab.target;
    match slot.lane {
        Lane::Hand => {
            if let Some(id) = slot.hand_card {
                let direction = if to < grab.origin {
                    MoveDirection::Left
                } else {
                    MoveDirection::Right
                };
                for _ in 0..to.abs_diff(grab.origin) {
                    clicks.write(Click::Play(Command::MoveCard { id, direction }));
                }
            }
        }
        Lane::Jokers => {
            clicks.write(Click::Play(Command::ReorderJoker {
                from: grab.origin,
                to,
                revision: grab.revision,
            }));
        }
        Lane::Consumables => {
            clicks.write(Click::Play(Command::ReorderConsumable {
                from: grab.origin,
                to,
                revision: grab.revision,
            }));
        }
    }
}

fn px_value(value: Val) -> f32 {
    if let Val::Px(value) = value {
        value
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests;

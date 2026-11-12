use crate::app::{
    navigation::Screen,
    session::{Click, Session},
};
use ante_core::contract::{CardKey, Command, MoveDirection};
use bevy::{
    ecs::system::EntityCommands,
    picking::prelude::{PointerButton, PointerOver, PointerPress},
    prelude::*,
    window::PrimaryWindow,
};

#[derive(Clone, Copy, PartialEq, Eq)]
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

pub(crate) fn update(
    mut drag: ResMut<CardDrag>,
    session: Res<Session>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut slots: Query<(
        Entity,
        &mut Slot,
        &ComputedNode,
        &UiGlobalTransform,
        &mut UiTransform,
        &mut GlobalZIndex,
    )>,
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
    let Ok((_, slot, node, global, mut transform, mut z)) = slots.get_mut(grab.entity) else {
        drag.grab = None;
        return;
    };
    let slot = *slot;
    let lane = slot.lane;
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
            if grab.moved
                && let Some(inverse) = global.try_inverse()
            {
                let shift = inverse.transform_vector2(delta)
                    * grab.transform.scale
                    * node.inverse_scale_factor();
                let px_value = |value| {
                    if let Val::Px(value) = value {
                        value
                    } else {
                        0.0
                    }
                };
                transform.translation.x = px(px_value(grab.transform.translation.x) + shift.x);
                transform.translation.y = px(px_value(grab.transform.translation.y) + shift.y);
                // Keep the card in its current stack position while it follows the cursor.
                z.0 = grab.z;
            }
            if grab.moved {
                let desired_x = grab.center.x + delta.x;
                let neighbor = slots
                    .iter()
                    .filter(|(entity, other, ..)| {
                        *entity != grab.entity
                            && other.lane == slot.lane
                            && (other.index as isize - grab.target as isize).abs() == 1
                    })
                    .min_by(|(_, _, _, a, ..), (_, _, _, b, ..)| {
                        (a.translation.x - desired_x)
                            .abs()
                            .total_cmp(&(b.translation.x - desired_x).abs())
                    })
                    .map(|(entity, other, node, global, ..)| {
                        (entity, other.index, node.size().x, global.translation.x)
                    });
                if let Some((entity, neighbor_index, width, neighbor_x)) = neighbor {
                    let crossed = if neighbor_index > grab.target {
                        desired_x > neighbor_x
                    } else {
                        desired_x < neighbor_x
                    };
                    if crossed
                        && let Ok((_, mut other, _, _, mut other_transform, _)) =
                            slots.get_mut(entity)
                    {
                        let sign = if neighbor_index > grab.target {
                            -1.0
                        } else {
                            1.0
                        };
                        let x = match other_transform.translation.x {
                            Val::Px(value) => value,
                            _ => 0.0,
                        };
                        other_transform.translation.x = px(x + sign * width);
                        let old_index = grab.target;
                        other.index = old_index;
                        if let Ok((_, mut dragged, _, _, _, _)) = slots.get_mut(grab.entity) {
                            dragged.index = neighbor_index;
                        }
                        grab.target = neighbor_index;
                        for (entity, slot, _, _, _, mut layer) in &mut slots {
                            if slot.lane == lane {
                                layer.0 = slot.index as i32;
                            }
                            if entity == grab.entity {
                                layer.0 = grab.z;
                            }
                        }
                    }
                }
            }
            return;
        }
        if !grab.moved {
            if let Some(click) = slot.click {
                clicks.write(click);
            }
        } else {
            let to = grab.target;
            if to != grab.origin {
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
        }
    } else if valid && window.is_some() && mouse.pressed(MouseButton::Left) {
        return;
    }
    *transform = if grab.z > 0 {
        grab.transform
    } else {
        UiTransform::IDENTITY
    };
    z.0 = grab.z;
    drag.grab = None;
}

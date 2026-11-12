//! Jokers and consumables that leave the inventory burn away like the original
//! `Card:start_dissolve`: 0.7 s, the dissolve shader driven from 0 to 1, burn colours,
//! particles and the whoosh and crumple sounds. A sold card burns in gold.

use ante_core::contract::{Command as MvpAction, JokerKey};
use bevy::{prelude::*, ui::UiGlobalTransform};

use crate::{
    app::{
        presentation::ScoredJoker,
        session::{Click, Session},
    },
    components::{
        edition::{EditionOverlay, LoveMaterial},
        tiles::TILE,
        travel::pin_in_place,
    },
    services::assets::{
        shaders::LoveShaders,
        sounds::{Sfx, jitter},
    },
};

/// Original `start_dissolve` duration.
const DISSOLVE_TIME: f32 = 0.7;
const GOLD: Color = Color::srgb(
    0xEA as f32 / 255.0,
    0xC0 as f32 / 255.0,
    0x58 as f32 / 255.0,
);
/// Default dissolve colours: black, orange, red, gold and joker grey.
const DEFAULT_COLOURS: [Color; 5] = [
    Color::srgb(
        0x37 as f32 / 255.0,
        0x42 as f32 / 255.0,
        0x44 as f32 / 255.0,
    ),
    Color::srgb(
        0xFD as f32 / 255.0,
        0xA2 as f32 / 255.0,
        0x00 as f32 / 255.0,
    ),
    Color::srgb(
        0xFE as f32 / 255.0,
        0x5F as f32 / 255.0,
        0x55 as f32 / 255.0,
    ),
    GOLD,
    Color::srgb(
        0xBF as f32 / 255.0,
        0xC7 as f32 / 255.0,
        0xD5 as f32 / 255.0,
    ),
];

/// Consumable shown in the inventory, by its position there.
#[derive(Component)]
pub(crate) struct HeldConsumable {
    pub(crate) index: usize,
}

/// Use and sell buttons beside a selected inventory card; they leave with the card.
#[derive(Component)]
pub(crate) struct InventoryActions;

/// A detached card burning away.
#[derive(Component)]
pub(crate) struct Dissolving {
    elapsed: f32,
    colours: Vec<Color>,
    started: bool,
}

impl Dissolving {
    fn new(colours: &[Color]) -> Self {
        Self {
            elapsed: 0.0,
            colours: colours.to_vec(),
            started: false,
        }
    }

    fn progress(&self) -> f32 {
        (self.elapsed / DISSOLVE_TIME).min(1.0)
    }
}

#[derive(Component)]
pub(crate) struct Particle {
    velocity: Vec2,
    offset: Vec2,
    age: f32,
}

type Held<'a> = (
    Entity,
    &'a mut Node,
    &'a ComputedNode,
    &'a UiGlobalTransform,
    &'a UiTransform,
);

/// After the session changed, detaches the Jokers and consumables that left the
/// inventory so they outlive the rebuilt board and burn where they were.
#[allow(clippy::type_complexity)]
pub(crate) fn detach_leaving(
    mut commands: Commands,
    mut clicks: MessageReader<Click>,
    session: Res<Session>,
    mut previous: Local<Option<(u64, Vec<JokerKey>, Vec<&'static str>)>>,
    mut jokers: Query<(&ScoredJoker, Held), Without<HeldConsumable>>,
    mut consumables: Query<(&HeldConsumable, Held), Without<ScoredJoker>>,
) {
    let sold = clicks.read().any(|click| {
        matches!(
            click,
            Click::Play(MvpAction::SellJoker { .. } | MvpAction::SellConsumable { .. })
        )
    });
    if !session.is_changed() {
        return;
    }
    let Some(view) = session.confirmed.as_ref() else {
        *previous = None;
        return;
    };
    let held: Vec<JokerKey> = view.held_jokers.iter().map(|joker| joker.id).collect();
    let kinds: Vec<&'static str> = view.consumables.iter().map(|card| card.kind).collect();
    if let Some((run, before_jokers, before_kinds)) = previous.as_ref()
        && *run == session.run_id
    {
        // `sell_card` burns in gold; other removals use the default colours.
        let colours: &[Color] = if sold { &[GOLD] } else { &DEFAULT_COLOURS };
        for (joker, (entity, mut node, computed, global, transform)) in &mut jokers {
            if before_jokers.contains(&joker.id) && !held.contains(&joker.id) {
                pin_in_place(&mut node, computed, global, transform);
                burn(&mut commands, entity, colours);
            }
        }
        if kinds.len() < before_kinds.len() {
            let gone = before_kinds
                .iter()
                .zip(kinds.iter().map(Some).chain(std::iter::repeat(None)))
                .position(|(before, now)| now != Some(before))
                .unwrap_or(before_kinds.len() - 1);
            for (card, (entity, mut node, computed, global, transform)) in &mut consumables {
                if card.index == gone {
                    pin_in_place(&mut node, computed, global, transform);
                    burn(&mut commands, entity, colours);
                }
            }
        }
    }
    *previous = Some((session.run_id, held, kinds));
}

fn burn(commands: &mut Commands, entity: Entity, colours: &[Color]) {
    commands.entity(entity).remove::<ChildOf>().insert((
        Dissolving::new(colours),
        GlobalZIndex(400),
        Pickable::IGNORE,
    ));
}

/// Drives the dissolve: shader overlays on every sprite, the shared `dissolve` value and
/// burn colours on every shader of the card, then removal at 1.05 × the duration.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(crate) fn dissolve(
    mut commands: Commands,
    time: Res<Time<Real>>,
    shaders: Option<Res<LoveShaders>>,
    mut cards: Query<(Entity, &mut Dissolving, &ComputedNode)>,
    children: Query<&Children>,
    mut images: Query<&mut ImageNode, Without<EditionOverlay>>,
    (overlays, actions): (
        Query<&MaterialNode<LoveMaterial>>,
        Query<(), With<InventoryActions>>,
    ),
    mut materials: ResMut<Assets<LoveMaterial>>,
    mut sounds: MessageWriter<Sfx>,
) {
    let shader = shaders
        .as_ref()
        .is_some_and(|shaders| shaders.get("dissolve").is_some());
    for (entity, mut card, node) in &mut cards {
        if !card.started {
            card.started = true;
            sounds.write(Sfx::new("whoosh2", jitter(0.9, 0.2), 0.5));
            let crumple = ["crumple1", "crumple2", "crumple3", "crumple4", "crumple5"]
                [(rand::random::<f32>() * 5.0) as usize % 5];
            sounds.write(Sfx::new(crumple, jitter(0.9, 0.2), 0.5));
            for child in children.iter_descendants(entity) {
                if actions.contains(child) {
                    commands.entity(child).despawn();
                    continue;
                }
                if let Ok(image) = images.get(child)
                    && shader
                {
                    let sprite = image.clone();
                    commands.entity(child).with_children(|layer| {
                        super::edition::overlay(layer, "Dissolve", &sprite, 0);
                    });
                }
            }
            let size = node.size() * node.inverse_scale_factor();
            commands.entity(entity).with_children(|parts| {
                for index in 0..24 {
                    spawn_particle(parts, size, card.colours[index % card.colours.len()]);
                }
            });
        }
        card.elapsed += time.delta_secs();
        let progress = card.progress();
        let burn = |index: usize| {
            card.colours.get(index).map_or(Vec4::ZERO, |colour| {
                Vec4::from(colour.to_srgba().to_f32_array())
            })
        };
        let (first, second) = (burn(0), burn(1));
        for child in children.iter_descendants(entity) {
            if let Ok(overlay) = overlays.get(child)
                && let Some(mut material) = materials.get_mut(&overlay.0)
            {
                material.set("dissolve", Vec4::splat(progress));
                material.set("burn_colour_1", first);
                material.set("burn_colour_2", second);
            }
            if let Ok(mut image) = images.get_mut(child) {
                // The sprite hides only once its dissolve layer draws, so it never blinks.
                let covered = children
                    .get(child)
                    .is_ok_and(|layers| layers.iter().any(|layer| overlays.contains(layer)));
                let alpha = match (shader, covered) {
                    (true, true) => 0.0,
                    (true, false) => 1.0,
                    (false, _) => 1.0 - progress,
                };
                if image.color.alpha() != alpha {
                    image.color = image.color.with_alpha(alpha);
                }
            }
        }
        if card.elapsed >= 1.05 * DISSOLVE_TIME {
            commands.entity(entity).despawn();
        }
    }
}

/// Small squares like the original `Particles` (scale 0.1, speed 2, lifespan 0.7 × 0.7).
fn spawn_particle(parts: &mut ChildSpawnerCommands, size: Vec2, colour: Color) {
    let at = Vec2::new(rand::random::<f32>(), rand::random::<f32>()) * size;
    let angle = rand::random::<f32>() * std::f32::consts::TAU;
    let speed = 2.0 * TILE * (0.5 + rand::random::<f32>());
    parts.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(at.x),
            top: px(at.y),
            width: px(0.1 * TILE),
            height: px(0.1 * TILE),
            ..default()
        },
        BackgroundColor(colour),
        Particle {
            velocity: Vec2::from_angle(angle) * speed,
            offset: Vec2::ZERO,
            age: 0.0,
        },
        Pickable::IGNORE,
    ));
}

pub(crate) fn move_particles(
    time: Res<Time<Real>>,
    mut particles: Query<(&mut Particle, &mut UiTransform, &mut BackgroundColor)>,
) {
    let dt = time.delta_secs();
    let life = 0.7 * DISSOLVE_TIME;
    for (mut particle, mut transform, mut colour) in &mut particles {
        particle.age += dt;
        let velocity = particle.velocity;
        particle.offset += velocity * dt;
        let fade = (1.0 - particle.age / life).clamp(0.0, 1.0);
        transform.translation = Val2::new(Val::Px(particle.offset.x), Val::Px(particle.offset.y));
        transform.scale = Vec2::splat(fade);
        colour.0 = colour.0.with_alpha(fade);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dissolve_runs_for_the_original_duration() {
        let mut card = Dissolving::new(&DEFAULT_COLOURS);
        card.elapsed = 0.35;
        assert!((card.progress() - 0.5).abs() < 1e-6);
        card.elapsed = 2.0;
        assert_eq!(card.progress(), 1.0);
    }
}

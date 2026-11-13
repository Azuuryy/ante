//! Booster pack opening, after `Game:update_arcana_pack` and `update_spectral_pack`: the
//! `booster_pack_sparkles` particles filling the room and the pack box rising from below
//! the hand. Presentation only; the random values never touch the simulation RNG.

use bevy::{picking::prelude::Pickable, prelude::*};
use rand::Rng;

use super::tiles::{CANVAS, GOLD, ROOM, TILE, lighten};
use crate::app::session::Session;
use ante_core::contract::{PackFamily, Phase};

/// `Particles` settings for one pack family.
#[derive(Clone, PartialEq)]
struct Settings {
    /// Seconds between two particles.
    timer: f32,
    /// Largest particle side, in tiles.
    scale: f32,
    lifespan: f32,
    /// Starting speed in tiles per second.
    speed: f32,
    colours: Vec<Color>,
}

/// `G.C.PURPLE`.
const PURPLE: Color = Color::srgb(
    0x88 as f32 / 255.0,
    0x67 as f32 / 255.0,
    0xA5 as f32 / 255.0,
);

fn settings(family: PackFamily) -> Option<Settings> {
    match family {
        PackFamily::Arcana => Some(Settings {
            timer: 0.015,
            scale: 0.2,
            lifespan: 1.0,
            speed: 1.1,
            colours: vec![
                Color::WHITE,
                lighten(PURPLE, 0.4),
                lighten(PURPLE, 0.2),
                lighten(GOLD, 0.2),
            ],
        }),
        PackFamily::Spectral => Some(Settings {
            timer: 0.015,
            scale: 0.1,
            lifespan: 3.0,
            speed: 0.2,
            colours: vec![Color::WHITE, lighten(GOLD, 0.2)],
        }),
        _ => None,
    }
}

/// The particle layer over the room; `fade` is the original `fade_alpha` (1 = hidden).
#[derive(Component)]
pub(crate) struct SparkleLayer {
    settings: Option<Settings>,
    clock: f32,
    fade: f32,
}

#[derive(Component)]
pub(crate) struct Sparkle {
    age: f32,
    lifespan: f32,
    scale: f32,
    dir: f32,
    facing: f32,
    r_vel: f32,
    velocity: f32,
    offset: Vec2,
    colour: Color,
}

/// A layer over the background and under the board, spawned once with the run board.
pub(crate) fn layer() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: px(CANVAS.x),
            height: px(CANVAS.y),
            ..default()
        },
        SparkleLayer {
            settings: None,
            clock: 0.0,
            fade: 1.0,
        },
        Pickable::IGNORE,
    )
}

/// The pack box: it starts `ROOM.y + 9` tiles low and eases up to its place.
#[derive(Component)]
pub(crate) struct PackRise;

/// When the open pack appeared; rebuilds of the pack box keep the same motion.
#[derive(Resource, Default)]
pub(crate) struct PackOpening {
    since: Option<f32>,
}

/// Seconds the rise takes, standing in for the UIBox alignment ease.
const RISE_TIME: f32 = 0.45;
/// Seconds of the `fade(1, 0)` fade-in, and of the fade-out once the pack closes.
const FADE_TIME: f32 = 1.0;

pub(crate) fn update(
    mut commands: Commands,
    session: Res<Session>,
    time: Res<Time<Real>>,
    mut opening: ResMut<PackOpening>,
    mut layers: Query<(Entity, &mut SparkleLayer)>,
    mut sparkles: Query<(
        Entity,
        &mut Sparkle,
        &mut Node,
        &mut UiTransform,
        &mut BackgroundColor,
    )>,
    mut rising: Query<&mut UiTransform, (With<PackRise>, Without<Sparkle>)>,
) {
    let now = time.elapsed_secs();
    let dt = time.delta_secs();
    let open = session
        .confirmed
        .as_ref()
        .filter(|view| matches!(view.phase, Phase::Pack | Phase::Target))
        .and_then(|view| view.open_pack);
    // The pack box rise.
    match open {
        Some(_) => {
            let since = *opening.since.get_or_insert(now);
            let t = ((now - since) / RISE_TIME).clamp(0.0, 1.0);
            let ease = 1.0 - (1.0 - t).powi(3);
            let start = (ROOM.y / TILE + 9.0) * TILE;
            for mut transform in &mut rising {
                transform.translation = Val2::px(0.0, start * (1.0 - ease));
            }
        }
        None => opening.since = None,
    }
    let mut rng = rand::thread_rng();
    for (layer_entity, mut layer) in &mut layers {
        let wanted = open.and_then(settings);
        if wanted.is_some() && layer.settings != wanted {
            layer.settings = wanted.clone();
            layer.fade = 1.0;
            layer.clock = 0.0;
        }
        let emitting = wanted.is_some();
        // `fade(1, 0)` on opening, back to 1 when the pack closes.
        let step = dt / FADE_TIME;
        layer.fade = if emitting {
            (layer.fade - step).max(0.0)
        } else {
            (layer.fade + step).min(1.0)
        };
        let Some(settings) = layer.settings.clone() else {
            for (entity, ..) in &sparkles {
                commands.entity(entity).despawn();
            }
            continue;
        };
        if emitting {
            layer.clock += dt;
            let mut added = 0;
            while layer.clock > settings.timer && added < 20 {
                layer.clock -= settings.timer;
                added += 1;
                let offset = Vec2::new(
                    (0.5 - rng.r#gen::<f32>()) * ROOM.x,
                    (0.5 - rng.r#gen::<f32>()) * ROOM.y,
                );
                let colour = settings.colours[rng.gen_range(0..settings.colours.len())];
                commands.entity(layer_entity).with_children(|layer| {
                    layer.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(0),
                            top: px(0),
                            width: px(0),
                            height: px(0),
                            ..default()
                        },
                        UiTransform::IDENTITY,
                        BackgroundColor(Color::NONE),
                        Sparkle {
                            age: 0.0,
                            lifespan: settings.lifespan,
                            scale: settings.scale,
                            dir: rng.r#gen::<f32>() * std::f32::consts::TAU,
                            facing: rng.r#gen::<f32>() * std::f32::consts::TAU,
                            r_vel: 0.2 * (0.5 - rng.r#gen::<f32>()),
                            velocity: settings.speed * 0.7,
                            offset,
                            colour,
                        },
                        Pickable::IGNORE,
                    ));
                });
            }
        } else if layer.fade >= 1.0 {
            layer.settings = None;
        }
        let alpha = 1.0 - layer.fade;
        for (entity, mut sparkle, mut node, mut transform, mut colour) in &mut sparkles {
            // `Particles:move`: grow then shrink over the lifespan, slow down by 7 % a second.
            sparkle.age += dt;
            let side = (2.0
                * ((sparkle.age / sparkle.lifespan) * sparkle.scale)
                    .min(sparkle.scale * (sparkle.lifespan - sparkle.age) / sparkle.lifespan))
            .min(sparkle.scale);
            if side < 0.0 {
                commands.entity(entity).despawn();
                continue;
            }
            let step = sparkle.velocity * dt * TILE;
            let heading = Vec2::new(sparkle.dir.sin(), sparkle.dir.cos());
            sparkle.offset += heading * step;
            sparkle.facing += sparkle.r_vel * dt;
            sparkle.velocity = (sparkle.velocity - sparkle.velocity * 0.07 * dt).max(0.0);
            let side = side * TILE;
            let center = sparkle.offset + CANVAS / 2.0;
            node.left = px(center.x - side / 2.0);
            node.top = px(center.y - side / 2.0);
            node.width = px(side);
            node.height = px(side);
            transform.rotation = Rot2::radians(sparkle.facing);
            *colour = BackgroundColor(sparkle.colour.with_alpha(sparkle.colour.alpha() * alpha));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_arcana_and_spectral_packs_sparkle() {
        assert!(settings(PackFamily::Arcana).is_some());
        assert!(settings(PackFamily::Spectral).is_some());
        assert!(settings(PackFamily::Buffoon).is_none());
        assert_eq!(settings(PackFamily::Spectral).unwrap().lifespan, 3.0);
    }
}

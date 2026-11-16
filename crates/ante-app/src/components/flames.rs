//! Score flames: the original `flame` shader behind the Chips and Mult boxes once a hand's
//! Chips × Mult reaches the blind's score (`G.FUNCS.flame_handler`), and the intensity
//! that drives the fire and organ ambience (`modulate_sound`). Without the shader the boxes
//! stay plain.

use bevy::{prelude::*, ui_render::prelude::MaterialNode};

use super::{
    edition::LoveMaterial,
    tiles::{BLUE, RED, TILE},
};
use crate::{app::session::Session, services::assets::shaders::LoveShaders};
use ante_core::contract::Phase;

pub struct FlamesPlugin;

impl Plugin for FlamesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScoreIntensity>()
            .add_systems(Update, (attach_flames, feed_flames).chain());
    }
}

/// How hot the score burns, read by the ambience: `G.ARGS.score_intensity.flames` and
/// `.organ`, and the sum of both flames' `change`.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScoreIntensity {
    pub(crate) flames: f32,
    pub(crate) organ: f32,
    pub(crate) change: f32,
}

/// One flame's easing, as `G.ARGS.chip_flames` and `G.ARGS.mult_flames` keep it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct FlameState {
    pub(crate) real: f32,
    velocity: f32,
    pub(crate) change: f32,
    pub(crate) timer: f32,
}

impl FlameState {
    /// One frame of `flame_handler` toward `intensity`, with `dt` in real seconds.
    pub(crate) fn step(&mut self, intensity: f32, dt: f32) {
        self.timer += dt * (1.0 + intensity * 0.2);
        if self.velocity < 0.0 {
            self.velocity *= 1.0 - 10.0 * dt;
        }
        let exptime = (-0.4 * dt).exp();
        self.velocity =
            (1.0 - exptime) * (intensity - self.real) * dt * 25.0 + exptime * self.velocity;
        self.real = (self.real + self.velocity).max(0.0);
        let rising = if self.real < intensity {
            self.real
        } else {
            0.0
        };
        self.change = self.change * (1.0 - 4.0 * dt) + 4.0 * dt * rising;
    }
}

/// `flame_handler`'s target: `log₅(score) − 2` once the hand's Chips × Mult reaches the
/// blind's score, nothing while a pack is open.
pub(crate) fn flame_target(earned: f64, required: usize, pack_open: bool) -> f32 {
    if pack_open || required == 0 || earned < required as f64 {
        return 0.0;
    }
    (earned.log(5.0) - 2.0).max(0.0) as f32
}

/// `score_intensity.organ`: `0.1 × log₅(score / (blind + 1))`, between 0 and 0.4.
pub(crate) fn organ_target(earned: f64, required: usize) -> f32 {
    if required == 0 || earned <= 0.0 {
        return 0.0;
    }
    (0.1 * (earned / (required as f64 + 1.0)).log(5.0)).clamp(0.0, 0.4) as f32
}

/// A flame behind the Chips (`chips`) or Mult box.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct ScoreFlame {
    chips: bool,
    id: f32,
    state: FlameState,
}

/// The flame node: a 2.5 × 2.5 unit sprite aligned to the bottom middle inside its box
/// (`type = 'bmi'`), drawn before the box's number.
pub(crate) fn flame_layer(chips: bool) -> impl Bundle {
    let side = 2.5 * TILE;
    (
        ScoreFlame {
            chips,
            id: rand::random::<f32>() * 1000.0,
            state: FlameState::default(),
        },
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            bottom: px(0),
            width: px(side),
            height: px(side),
            ..default()
        },
        UiTransform::from_translation(Val2::px(-side / 2.0, 0.0)),
        Visibility::Hidden,
        Pickable::IGNORE,
    )
}

fn attach_flames(
    mut commands: Commands,
    flames: Query<(Entity, &ScoreFlame), Without<MaterialNode<LoveMaterial>>>,
    shaders: Option<Res<LoveShaders>>,
    mut materials: ResMut<Assets<LoveMaterial>>,
) {
    let Some((shader, uniforms)) = shaders.as_ref().and_then(|s| s.get("flame")) else {
        return;
    };
    for (entity, flame) in &flames {
        let mut material =
            LoveMaterial::new(shader.clone(), uniforms.clone(), Handle::default(), "flame");
        // The shader only reads the coordinates within its cell: the whole node is one.
        material.set("texture_details", Vec4::new(0.0, 0.0, 1.0, 1.0));
        material.set("image_details", Vec4::new(1.0, 1.0, 0.0, 0.0));
        material.set("id", Vec4::splat(flame.id));
        let (colour, accent) = flame_colours(flame.chips);
        material.set("colour_1", colour);
        material.set("colour_2", accent);
        commands
            .entity(entity)
            .insert(MaterialNode(materials.add(material)));
    }
}

/// `UI_CHIPS` or `UI_MULT`, and its lick: `clamp(((c + YELLOW) / 2 + 0.1)², 0.1, 1)`.
fn flame_colours(chips: bool) -> (Vec4, Vec4) {
    let base = if chips { BLUE } else { RED }.to_srgba();
    let yellow = [1.0, 1.0, 0.0];
    let lick = |c: f32, y: f32| ((c * 0.5 + y * 0.5) + 0.1).powi(2).clamp(0.1, 1.0);
    (
        Vec4::new(base.red, base.green, base.blue, 1.0),
        Vec4::new(
            lick(base.red, yellow[0]),
            lick(base.green, yellow[1]),
            lick(base.blue, yellow[2]),
            1.0,
        ),
    )
}

/// The score the HUD shows (`current_hand.chips × mult`: the trace's values, or the
/// selected hand's preview), the blind's score, and whether a pack is open.
fn scores(session: &Session) -> (f64, usize, bool) {
    let Some(view) = session.confirmed.as_ref() else {
        return (0.0, 0, false);
    };
    let pack = view.phase == Phase::Pack;
    let ((chips, mult), required) = match &session.playback {
        Some((before, playback)) => (playback.score_values().unwrap_or((0, 0.0)), before.target),
        None if view.phase == Phase::Playing => (
            view.selected_hand
                .map_or((0, 0.0), |hand| (hand.level.chips, hand.level.mult as f64)),
            view.target,
        ),
        None => ((0, 0.0), 0),
    };
    (chips as f64 * mult, required, pack)
}

fn feed_flames(
    time: Res<Time<Real>>,
    session: Res<Session>,
    mut intensity: ResMut<ScoreIntensity>,
    mut flames: Query<(
        &mut ScoreFlame,
        &mut Visibility,
        Option<&MaterialNode<LoveMaterial>>,
    )>,
    mut materials: ResMut<Assets<LoveMaterial>>,
) {
    let dt = time.delta_secs();
    let (earned, required, pack) = scores(&session);
    let target = flame_target(earned, required, pack);
    let mut chips_flame = None;
    let mut change = 0.0;
    for (mut flame, mut visibility, material) in &mut flames {
        flame.state.step(target, dt);
        change += flame.state.change;
        if flame.chips {
            chips_flame = Some(flame.state);
        }
        let shown = if flame.state.real > 0.001 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        visibility.set_if_neq(shown);
        if let Some(mut material) = material.and_then(|node| materials.get_mut(&node.0)) {
            material.set("time", Vec4::splat(flame.state.timer));
            material.set("amount", Vec4::splat(flame.state.real));
        }
    }
    let flames = chips_flame.map_or(0.0, |flame| ((flame.real + flame.change) / 10.0).min(1.0));
    let next = ScoreIntensity {
        flames,
        organ: organ_target(earned, required),
        change,
    };
    if *intensity != next {
        *intensity = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flames_burn_once_the_hand_reaches_the_blind() {
        assert_eq!(flame_target(250.0, 300, false), 0.0);
        assert_eq!(
            flame_target(300.0, 300, true),
            0.0,
            "an open pack has no flames"
        );
        // log₅(3125) − 2 = 3.
        assert!((flame_target(3125.0, 300, false) - 3.0).abs() < 1e-4);
        // A single hand past the blind already burns: log₅(301) − 2 ≈ 1.55.
        assert!((flame_target(301.0, 300, false) - 1.546).abs() < 1e-3);
        assert_eq!(flame_target(20.0, 0, false), 0.0, "no blind, no flame");
    }

    #[test]
    fn the_flame_eases_up_then_dies_down() {
        let mut flame = FlameState::default();
        for _ in 0..120 {
            flame.step(3.0, 1.0 / 60.0);
        }
        assert!(flame.real > 1.0, "{}", flame.real);
        assert!(flame.change > 0.0);
        for _ in 0..600 {
            flame.step(0.0, 1.0 / 60.0);
        }
        assert!(flame.real < 0.05, "{}", flame.real);
    }

    #[test]
    fn the_organ_rises_with_the_margin_over_the_blind() {
        assert_eq!(organ_target(100.0, 300), 0.0);
        assert_eq!(organ_target(1.0e12, 300), 0.4);
        assert!(organ_target(30_000.0, 300) > 0.0);
    }

    #[test]
    fn the_flame_lick_mixes_the_colour_with_yellow() {
        let (colour, accent) = flame_colours(true);
        assert!(colour.z > 0.9, "chips flames are blue");
        assert!(accent.y > accent.z, "the lick leans to yellow");
    }
}

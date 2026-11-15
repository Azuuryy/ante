//! The run background: the original `background` shader from the local installation, fed
//! with the colours of `ease_background_colour_blind`. Without the shader the layer keeps a
//! flat fill of the same base colour.

use bevy::{prelude::*, ui_render::prelude::MaterialNode};

use super::{
    edition::LoveMaterial,
    pixel_rect::boss_colour,
    tiles::{BLACK, BLUE, IMPORTANT, RED, darken, lighten, mix},
};
use crate::{app::session::Session, services::assets::shaders::LoveShaders};
use ante_core::contract::{Phase, ShopArt};

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BackgroundColours>()
            .add_systems(Update, (ease_colours, attach_material, feed_shader).chain());
    }
}

/// The full-canvas layer behind the room.
#[derive(Component)]
pub(crate) struct BackgroundLayer;

pub(crate) fn layer() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: percent(100),
            height: percent(100),
            ..default()
        },
        BackgroundColor(Color::srgb(0.31, 0.52, 0.43)),
        BackgroundLayer,
        Pickable::IGNORE,
    )
}

/// `G.C.BACKGROUND`: C, L, D and the contrast.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Palette {
    pub(crate) c: Vec4,
    pub(crate) l: Vec4,
    pub(crate) d: Vec4,
    pub(crate) contrast: f32,
}

#[derive(Resource, Clone, Copy)]
struct BackgroundColours {
    current: Palette,
    target: Palette,
}

impl Default for BackgroundColours {
    fn default() -> Self {
        let start = palette(Phase::BeforeBlind, None, None);
        Self {
            current: start,
            target: start,
        }
    }
}

fn v(color: Color) -> Vec4 {
    let c = color.to_srgba();
    Vec4::new(c.red, c.green, c.blue, 1.0)
}

/// `ease_background_colour`: with only a new colour, L, D and C are 1.3, 0.7 and 0.9 of it;
/// a special colour becomes C (and D is 0.4); a tertiary one becomes D with L the new colour.
fn ease_args(
    new: Color,
    special: Option<Color>,
    tertiary: Option<Color>,
    contrast: f32,
) -> Palette {
    let new = v(new);
    let scaled = |k: f32| (new.truncate() * k).extend(1.0);
    match (special, tertiary) {
        (Some(special), Some(tertiary)) => Palette {
            c: v(special),
            l: new,
            d: v(tertiary),
            contrast,
        },
        (Some(special), None) => Palette {
            c: v(special),
            l: scaled(1.3),
            d: scaled(0.4),
            contrast,
        },
        _ => Palette {
            c: scaled(0.9),
            l: scaled(1.3),
            d: scaled(0.7),
            contrast,
        },
    }
}

/// Background for a phase and blind, after `ease_background_colour_blind`.
pub(crate) fn palette(phase: Phase, blind: Option<&str>, pack: Option<ShopArt>) -> Palette {
    let small = Srgba::hex("50846e").map(Color::from).unwrap_or(BLACK);
    match phase {
        Phase::Pack => match pack {
            Some(ShopArt::Card { .. }) => ease_args(darken(BLACK, 0.2), Some(RED), None, 3.0),
            Some(ShopArt::Joker(_)) => ease_args(IMPORTANT, Some(BLACK), None, 2.0),
            _ => ease_args(
                Srgba::hex("8867a5").map(Color::from).unwrap_or(BLACK),
                Some(darken(BLACK, 0.2)),
                None,
                1.5,
            ),
        },
        _ => match blind {
            Some(id) if id.starts_with("bl_final_") => {
                ease_args(BLUE, Some(RED), Some(darken(BLACK, 0.4)), 3.0)
            }
            Some(id) => match boss_colour(id) {
                Some(boss) => ease_args(lighten(mix(boss, BLACK, 0.3), 0.1), Some(boss), None, 2.0),
                None => ease_args(small, None, None, 1.0),
            },
            None => ease_args(small, None, None, 1.0),
        },
    }
}

fn ease_colours(
    session: Res<Session>,
    time: Res<Time<Real>>,
    mut colours: ResMut<BackgroundColours>,
) {
    if let Some(view) = session.confirmed.as_ref() {
        let blind = view.blind_chip;
        let pack = view.pack_choices.first().map(|item| item.art);
        colours.target = palette(view.phase, blind, pack);
    }
    // `ease_value` over 0.6 game seconds.
    let blend = 1.0 - (-time.delta_secs() * 5.0).exp();
    let target = colours.target;
    let current = &mut colours.current;
    current.c = current.c.lerp(target.c, blend);
    current.l = current.l.lerp(target.l, blend);
    current.d = current.d.lerp(target.d, blend);
    current.contrast += (target.contrast - current.contrast) * blend;
}

fn attach_material(
    mut commands: Commands,
    layers: Query<Entity, (With<BackgroundLayer>, Without<MaterialNode<LoveMaterial>>)>,
    shaders: Option<Res<LoveShaders>>,
    mut materials: ResMut<Assets<LoveMaterial>>,
) {
    let Some((shader, uniforms)) = shaders.as_ref().and_then(|s| s.get("background")) else {
        return;
    };
    for entity in &layers {
        let material = LoveMaterial::new(
            shader.clone(),
            uniforms.clone(),
            Handle::default(),
            "background",
        );
        commands
            .entity(entity)
            .insert(MaterialNode(materials.add(material)));
    }
}

fn feed_shader(
    time: Res<Time<Real>>,
    colours: Res<BackgroundColours>,
    windows: Query<&Window>,
    mut layers: Query<
        (&mut BackgroundColor, Option<&MaterialNode<LoveMaterial>>),
        With<BackgroundLayer>,
    >,
    mut materials: ResMut<Assets<LoveMaterial>>,
) {
    let palette = colours.current;
    let screen = windows
        .iter()
        .next()
        .map(|window| window.physical_size().as_vec2())
        .unwrap_or(Vec2::ONE);
    for (mut fill, material) in &mut layers {
        let c = palette.c;
        fill.0 = Color::srgb(c.x, c.y, c.z);
        let Some(mut material) = material.and_then(|node| materials.get_mut(&node.0)) else {
            continue;
        };
        // The run never spins (`G.ARGS.spin.amount` = 0), so `spin_time` stays 0.
        material.set("time", Vec4::splat(time.elapsed_secs()));
        material.set("spin_time", Vec4::ZERO);
        material.set("colour_1", palette.c);
        material.set("colour_2", palette.l);
        material.set("colour_3", palette.d);
        material.set("contrast", Vec4::splat(palette.contrast));
        material.set("spin_amount", Vec4::ZERO);
        material.set("love_ScreenSize", screen.extend(1.0).extend(1.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_blinds_scale_the_green() {
        let p = palette(Phase::Playing, Some("bl_small"), None);
        let base = v(Srgba::hex("50846e").unwrap().into());
        assert!((p.l.truncate() - base.truncate() * 1.3).length() < 1e-4);
        assert!((p.d.truncate() - base.truncate() * 0.7).length() < 1e-4);
        assert_eq!(p.contrast, 1.0);
    }

    #[test]
    fn bosses_use_their_colour_as_c() {
        let p = palette(Phase::Playing, Some("bl_ox"), None);
        assert!((p.c - v(boss_colour("bl_ox").unwrap())).length() < 1e-4);
        assert_eq!(p.contrast, 2.0);
    }
}

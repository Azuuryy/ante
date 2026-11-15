//! Boxes drawn like the original UI (`UIElement:draw_pixellated_rect`): stepped corners, an
//! emboss lip in the darkened colour and an optional drop shadow, plus the dynamic UI colours
//! of `G.C.DYN_UI` that follow the phase and the blind.

use bevy::{
    asset::embedded_asset,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
    ui::ComputedNode,
    ui_render::{UiMaterialPlugin, prelude::MaterialNode, prelude::UiMaterial},
};

use super::tiles::{BLACK, BLUE, ORANGE, RED, TILE, darken, lighten, mix};
use crate::app::session::Session;
use ante_core::contract::{Phase, ShopArt};

pub struct PixelRectPlugin;

impl Plugin for PixelRectPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "pixel_rect.wgsl");
        app.add_plugins(UiMaterialPlugin::<PixelRect>::default())
            .init_resource::<DynUi>()
            .add_systems(
                Update,
                (
                    ease_dyn_ui,
                    apply_dyn_roles,
                    attach_materials,
                    sync_materials,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, ShaderType)]
struct PixelParams {
    fill: Vec4,
    emboss: Vec4,
    shadow: Vec4,
    geom: Vec4,
    outline: Vec4,
    // outline width in physical pixels
    line: Vec4,
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct PixelRect {
    #[uniform(0)]
    params: PixelParams,
}

impl UiMaterial for PixelRect {
    fn fragment_shader() -> ShaderRef {
        "embedded://ante_app/components/pixel_rect.wgsl".into()
    }
}

/// A box drawn with the original corners. `emboss` is the lip height in room units (0.05 in
/// most HUD boxes); `shadow` adds the button shadow.
#[derive(Component, Clone, Copy)]
pub(crate) struct PixelBox {
    pub(crate) fill: Color,
    pub(crate) emboss: f32,
    pub(crate) shadow: bool,
    /// Outline colour and width in logical pixels (`outline` in the original config).
    pub(crate) outline: Option<(Color, f32)>,
}

impl PixelBox {
    pub(crate) fn new(fill: Color) -> Self {
        Self {
            fill,
            emboss: 0.0,
            shadow: false,
            outline: None,
        }
    }

    pub(crate) fn outline(mut self, color: Color, width: f32) -> Self {
        self.outline = Some((color, width));
        self
    }

    pub(crate) fn emboss(mut self, emboss: f32) -> Self {
        self.emboss = emboss;
        self
    }

    pub(crate) fn shadow(mut self) -> Self {
        self.shadow = true;
        self
    }
}

/// The `G.C.DYN_UI` colour a box takes; it follows the phase like the original eases it.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum DynRole {
    Main,
    Dark,
    BossMain,
    BossDark,
}

/// Current and target `G.C.DYN_UI` colours.
#[derive(Resource, Clone, Copy)]
pub(crate) struct DynUi {
    current: [Vec4; 4],
    target: [Vec4; 4],
}

impl Default for DynUi {
    fn default() -> Self {
        let black = srgb_vec(BLACK);
        Self {
            current: [black; 4],
            target: [black; 4],
        }
    }
}

impl DynUi {
    pub(crate) fn color(&self, role: DynRole) -> Color {
        let c = self.current[role as usize];
        Color::srgba(c.x, c.y, c.z, c.w)
    }
}

fn srgb_vec(color: Color) -> Vec4 {
    let c = color.to_srgba();
    Vec4::new(c.red, c.green, c.blue, c.alpha)
}

/// `boss_colour` of each Boss Blind in `G.P_BLINDS`.
pub(crate) fn boss_colour(id: &str) -> Option<Color> {
    let hex = match id {
        "bl_ox" => "b95b08",
        "bl_hook" => "a84024",
        "bl_mouth" => "ae718e",
        "bl_fish" => "3e85bd",
        "bl_club" => "b9cb92",
        "bl_manacle" => "575757",
        "bl_tooth" => "b52d2d",
        "bl_wall" => "8a59a5",
        "bl_house" => "5186a8",
        "bl_mark" => "6a3847",
        "bl_final_bell" => "009cfd",
        "bl_wheel" => "50bf7c",
        "bl_arm" => "6865f3",
        "bl_psychic" => "efc03c",
        "bl_goad" => "b95c96",
        "bl_water" => "c6e0eb",
        "bl_eye" => "4b71e4",
        "bl_plant" => "709284",
        "bl_needle" => "5c6e31",
        "bl_head" => "ac9db4",
        "bl_final_leaf" => "56a786",
        "bl_final_vessel" => "8a71e1",
        "bl_window" => "a9a295",
        "bl_serpent" => "439a4f",
        "bl_pillar" => "7e6752",
        "bl_flint" => "e56a2f",
        "bl_final_acorn" => "fda200",
        "bl_final_heart" => "ac3232",
        _ => return None,
    };
    Srgba::hex(hex).ok().map(Color::from)
}

/// `get_blind_main_colour`: the boss colour, or the Small and Big Blind mixes.
pub(crate) fn blind_main_colour(id: &str) -> Color {
    boss_colour(id).unwrap_or(match id {
        "bl_small" => mix(BLUE, BLACK, 0.6),
        "bl_big" => mix(ORANGE, BLACK, 0.6),
        _ => BLACK,
    })
}

/// Target colours for a phase, after `ease_background_colour_blind` and
/// `Blind:change_colour`. `previous` keeps what the original leaves untouched.
pub(crate) fn dyn_targets(
    phase: Phase,
    blind: Option<&str>,
    pack: Option<ShopArt>,
    previous: [Color; 4],
) -> [Color; 4] {
    let [mut main, mut dark, mut boss_main, mut boss_dark] = previous;
    match phase {
        Phase::Shop => main = mix(RED, BLACK, 0.9),
        Phase::Pack => {
            main = match pack {
                Some(ShopArt::Card { .. }) => RED,
                Some(ShopArt::Joker(_)) => super::tiles::IMPORTANT,
                _ => mix(Color::WHITE, BLACK, 0.9),
            }
        }
        Phase::Playing => {
            if let Some(id) = blind {
                let colour = blind_main_colour(id);
                main = colour;
                dark = mix(colour, BLACK, 0.4);
                if boss_colour(id).is_some() {
                    boss_main = colour;
                    boss_dark = mix(colour, BLACK, 0.2);
                } else {
                    boss_main = darken(BLACK, 0.05);
                    boss_dark = lighten(BLACK, 0.07);
                }
            }
        }
        // Between blinds the boss colours are gone: the HUD is back to the plain greys.
        _ => {
            main = blind_main_colour(blind.unwrap_or("bl_small"));
            if blind.is_some_and(|id| boss_colour(id).is_some()) {
                main = mix(BLUE, BLACK, 0.6);
            }
            dark = mix(main, BLACK, 0.4);
            boss_main = darken(BLACK, 0.05);
            boss_dark = lighten(BLACK, 0.07);
        }
    }
    [main, dark, boss_main, boss_dark]
}

fn ease_dyn_ui(session: Res<Session>, time: Res<Time<Real>>, mut dyn_ui: ResMut<DynUi>) {
    if let Some(view) = session.confirmed.as_ref() {
        let previous = dyn_ui.target.map(|c| Color::srgba(c.x, c.y, c.z, c.w));
        let pack = view.pack_choices.first().map(|item| item.art);
        let blind = view
            .blind_chip
            .or_else(|| view.blinds.iter().find(|b| b.selectable).map(|b| b.id));
        let targets = dyn_targets(view.phase, blind, pack, previous);
        let targets = targets.map(srgb_vec);
        if targets != dyn_ui.target {
            dyn_ui.target = targets;
        }
    }
    // `ease_colour` reaches the new colour over a short real-time ease.
    let blend = 1.0 - (-time.delta_secs() * 8.0).exp();
    let target = dyn_ui.target;
    let mut moved = false;
    for (current, target) in dyn_ui.current.iter_mut().zip(target) {
        if current.distance_squared(target) > 1e-7 {
            *current = current.lerp(target, blend);
            moved = true;
        } else if *current != target {
            *current = target;
            moved = true;
        }
    }
    if !moved {
        dyn_ui.bypass_change_detection();
    }
}

fn apply_dyn_roles(dyn_ui: Res<DynUi>, mut boxes: Query<(Ref<DynRole>, &mut PixelBox)>) {
    for (role, mut pixel) in &mut boxes {
        if dyn_ui.is_changed() || role.is_added() {
            let colour = dyn_ui.color(*role);
            if pixel.fill != colour {
                pixel.fill = colour;
            }
        }
    }
}

fn attach_materials(
    mut commands: Commands,
    mut added: Query<(Entity, &PixelBox, &mut Node), Added<PixelBox>>,
    mut materials: ResMut<Assets<PixelRect>>,
) {
    for (entity, pixel, mut node) in &mut added {
        // The original draws the lip and shadow outside the box; here they are inside, so
        // the content keeps the visible body's centre.
        let below = pixel.emboss * TILE
            + if pixel.shadow {
                SHADOW_DROP * STEP
            } else {
                0.0
            };
        if below > 0.0 {
            node.padding.bottom = match node.padding.bottom {
                Val::Px(value) => Val::Px(value + below),
                _ => Val::Px(below),
            };
        }
        let handle = materials.add(PixelRect {
            params: PixelParams {
                fill: Vec4::ZERO,
                emboss: Vec4::ZERO,
                shadow: Vec4::ZERO,
                geom: Vec4::ZERO,
                outline: Vec4::ZERO,
                line: Vec4::ZERO,
            },
        });
        commands.entity(entity).insert(MaterialNode(handle));
    }
}

/// One room unit is 20 of these steps (`G.TILESIZE`).
const STEP: f32 = TILE / 20.0;
/// The button shadow falls 1.5 x 1.5 steps down (`shadow_parrallax.y` x parallax distance).
const SHADOW_DROP: f32 = 2.25;

/// Gamma used to turn the original (gamma-encoded) UI colours into the linear colours Bevy
/// blends. 2.2 matches LÖVE's plain gamma; raise it to darken the flat UI colours.
pub(crate) const UI_GAMMA: f32 = 2.2;

fn linear(color: Color) -> Vec4 {
    let c = color.to_srgba();
    Vec4::new(
        c.red.max(0.0).powf(UI_GAMMA),
        c.green.max(0.0).powf(UI_GAMMA),
        c.blue.max(0.0).powf(UI_GAMMA),
        c.alpha,
    )
}

type ChangedBox = Or<(
    Changed<PixelBox>,
    Changed<ComputedNode>,
    Added<MaterialNode<PixelRect>>,
)>;

fn sync_materials(
    boxes: Query<(&PixelBox, &ComputedNode, &MaterialNode<PixelRect>), ChangedBox>,
    mut materials: ResMut<Assets<PixelRect>>,
) {
    for (pixel, node, handle) in &boxes {
        let Some(mut material) = materials.get_mut(&handle.0) else {
            continue;
        };
        let unit = STEP / node.inverse_scale_factor().max(1e-6);
        let alpha = pixel.fill.alpha();
        material.params = PixelParams {
            fill: linear(pixel.fill),
            emboss: linear(darken(pixel.fill, 0.3)),
            shadow: if pixel.shadow {
                Vec4::new(0.0, 0.0, 0.0, 0.3 * alpha)
            } else {
                Vec4::ZERO
            },
            geom: Vec4::new(
                unit,
                pixel.emboss * 20.0,
                0.0,
                if pixel.shadow { SHADOW_DROP } else { 0.0 },
            ),
            outline: pixel.outline.map_or(Vec4::ZERO, |(color, _)| linear(color)),
            line: Vec4::splat(pixel.outline.map_or(0.0, |(_, width)| {
                width / node.inverse_scale_factor().max(1e-6)
            })),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Color, b: Color) -> bool {
        srgb_vec(a).distance(srgb_vec(b)) < 1e-3
    }

    #[test]
    fn the_shop_outline_is_mostly_red() {
        let previous = [BLACK; 4];
        let [main, ..] = dyn_targets(Phase::Shop, None, None, previous);
        assert!(close(main, mix(RED, BLACK, 0.9)));
    }

    #[test]
    fn small_blinds_keep_the_dark_hud() {
        let [main, dark, boss_main, boss_dark] =
            dyn_targets(Phase::Playing, Some("bl_small"), None, [BLACK; 4]);
        let blue = mix(BLUE, BLACK, 0.6);
        assert!(close(main, blue));
        assert!(close(dark, mix(blue, BLACK, 0.4)));
        assert!(close(boss_main, darken(BLACK, 0.05)));
        assert!(close(boss_dark, lighten(BLACK, 0.07)));
    }

    #[test]
    fn boss_blinds_tint_the_hud() {
        let [_, _, boss_main, _] = dyn_targets(Phase::Playing, Some("bl_ox"), None, [BLACK; 4]);
        assert!(close(boss_main, Srgba::hex("b95b08").unwrap().into()));
    }
}

//! Card editions drawn with the original shaders from the local installation.
//!
//! Like the original, the base sprite is drawn first and the edition shader draws over it
//! with the same atlas cell. Uniform values follow the original card draw: the effect
//! parameter is (real time / 28 + tilt, real time), `time` is a per-card constant and
//! `dissolve` stays at 0 outside of a sale.

use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, RenderPipelineDescriptor},
    shader::{Shader, ShaderRef},
    ui::UiGlobalTransform,
    ui_render::prelude::{MaterialNode, UiMaterial},
    ui_render::{UiMaterialPlugin, ui_material::UiMaterialKey},
};

use crate::services::assets::shaders::{LoveShaders, SLOTS, Uniform};

pub struct EditionPlugin;

impl Plugin for EditionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<LoveMaterial>::default())
            .add_systems(Update, (attach_materials, advance_time).chain());
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct LoveKey(Handle<Shader>);

impl From<&LoveMaterial> for LoveKey {
    fn from(material: &LoveMaterial) -> Self {
        Self(material.shader.clone())
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
#[bind_group_data(LoveKey)]
pub struct LoveMaterial {
    #[uniform(0)]
    slots: [Vec4; SLOTS],
    #[texture(1)]
    #[sampler(2)]
    image: Handle<Image>,
    shader: Handle<Shader>,
    uniforms: Vec<Uniform>,
    effect: String,
}

impl UiMaterial for LoveMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Default
    }

    fn specialize(descriptor: &mut RenderPipelineDescriptor, key: UiMaterialKey<Self>) {
        if let Some(fragment) = descriptor.fragment.as_mut() {
            fragment.shader = key.bind_group_data.0;
        }
    }
}

impl LoveMaterial {
    pub(crate) fn set(&mut self, name: &str, value: Vec4) {
        if let Some(uniform) = self.uniforms.iter().find(|uniform| uniform.name == name) {
            self.slots[uniform.slot] = value;
        }
    }
}

/// Requests an edition overlay over a sprite; a system turns it into a material node.
#[derive(Component, Clone)]
pub(crate) struct EditionOverlay {
    pub(crate) shader: &'static str,
    pub(crate) image: ImageNode,
    pub(crate) seed: usize,
}

/// Original shader names drawn over a card for an engine edition label.
pub(crate) fn edition_shaders(edition: &str) -> &'static [&'static str] {
    match edition {
        "Foil" => &["foil"],
        "Holographic" | "Holo" => &["holo"],
        "Polychrome" => &["polychrome"],
        "Negative" => &["negative", "negative_shine"],
        // Non-edition looks drawn the same way over a sprite.
        "Booster" => &["booster"],
        "Voucher" => &["voucher"],
        "Debuff" => &["debuff"],
        "Dissolve" => &["dissolve"],
        _ => &[],
    }
}

/// Spawns one full-size overlay node per edition shader over a sprite.
pub(crate) fn overlay(
    ui: &mut ChildSpawnerCommands,
    edition: &str,
    image: &ImageNode,
    seed: usize,
) {
    for &shader in edition_shaders(edition) {
        ui.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                ..default()
            },
            EditionOverlay {
                shader,
                image: image.clone(),
                seed,
            },
            Pickable::IGNORE,
        ));
    }
}

fn attach_materials(
    mut commands: Commands,
    overlays: Query<(Entity, &EditionOverlay), Without<MaterialNode<LoveMaterial>>>,
    shaders: Option<Res<LoveShaders>>,
    images: Res<Assets<Image>>,
    mut materials: ResMut<Assets<LoveMaterial>>,
) {
    for (entity, overlay) in &overlays {
        let Some((shader, uniforms)) = shaders.as_ref().and_then(|s| s.get(overlay.shader)) else {
            commands.entity(entity).remove::<EditionOverlay>();
            continue;
        };
        let Some(size) = images
            .get(&overlay.image.image)
            .map(|image| image.size_f32())
        else {
            continue;
        };
        let mut material = LoveMaterial {
            slots: [Vec4::ZERO; SLOTS],
            image: overlay.image.image.clone(),
            shader: shader.clone(),
            uniforms: uniforms.clone(),
            effect: overlay.shader.to_owned(),
        };
        let cell = overlay
            .image
            .rect
            .unwrap_or(Rect::from_corners(Vec2::ZERO, size));
        let cell_size = cell.size();
        material.set(
            "texture_details",
            Vec4::new(
                cell.min.x / cell_size.x,
                cell.min.y / cell_size.y,
                cell_size.x,
                cell_size.y,
            ),
        );
        material.set("image_details", size.extend(0.0).extend(0.0));
        material.set(
            "time",
            Vec4::splat((123.33412 * (overlay.seed as f32 / 1.14212)) % 3000.0),
        );
        commands
            .entity(entity)
            .insert(MaterialNode(materials.add(material)));
    }
}

/// The named parameter follows `Card:draw`: min(3 × rotation, 1) + real time / 28, real time.
fn advance_time(
    time: Res<Time<Real>>,
    overlays: Query<(&MaterialNode<LoveMaterial>, &UiGlobalTransform)>,
    mut materials: ResMut<Assets<LoveMaterial>>,
) {
    let real = time.elapsed_secs();
    for (node, global) in &overlays {
        let rotation = global.to_scale_angle_translation().1;
        let Some(mut material) = materials.get_mut(&node.0) else {
            continue;
        };
        let effect = material.effect.clone();
        // `dissolve` is drawn without a named parameter: that uniform is the dissolve amount.
        if effect == "dissolve" {
            continue;
        }
        let shine = (3.0 * rotation).min(1.0) + real / 28.0;
        material.set(&effect, Vec4::new(shine, real, 0.0, 0.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::assets::shaders::UniformKind;

    #[test]
    fn engine_editions_map_to_original_shaders() {
        assert_eq!(edition_shaders("Foil"), ["foil"]);
        assert_eq!(edition_shaders("Negative"), ["negative", "negative_shine"]);
        assert!(edition_shaders("Base").is_empty());
    }

    #[test]
    fn uniforms_land_in_their_slot() {
        let mut material = LoveMaterial {
            slots: [Vec4::ZERO; SLOTS],
            image: Handle::default(),
            shader: Handle::default(),
            uniforms: vec![Uniform {
                name: "foil".into(),
                slot: 3,
                kind: UniformKind::Vec2,
            }],
            effect: "foil".into(),
        };
        material.set("foil", Vec4::new(1.0, 2.0, 0.0, 0.0));
        material.set("missing", Vec4::ONE);
        assert_eq!(material.slots[3], Vec4::new(1.0, 2.0, 0.0, 0.0));
    }
}

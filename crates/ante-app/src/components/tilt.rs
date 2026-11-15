//! Hover tilt of cards, after the `position()` vertex stage of the original card shaders
//! (`hovering`, `mouse_screen_pos`, `screen_scale` sent by `Sprite:draw_shader`). Sprites
//! inside a card (an entity with `HoverMotion`) are drawn by `TiltImage`, and edition
//! overlays share the same vertex stage, so face and overlay lean together.

use bevy::{
    asset::embedded_asset,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
    ui_render::{UiMaterialPlugin, prelude::MaterialNode, prelude::UiMaterial},
    window::PrimaryWindow,
};

use super::{edition::LoveMaterial, tiles::TILE, widgets::HoverMotion};

pub struct TiltPlugin;

impl Plugin for TiltPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "tilt.wgsl");
        embedded_asset!(app, "tilt_image.wgsl");
        app.add_plugins(UiMaterialPlugin::<TiltImage>::default())
            .add_systems(
                Update,
                (tilt_card_sprites, root_overlays, update_tilt).chain(),
            );
    }
}

pub(crate) const TILT_SHADER: &str = "embedded://ante_app/components/tilt.wgsl";

/// Uniforms of the tilt vertex stage, in physical pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq, ShaderType)]
pub(crate) struct TiltParams {
    pub(crate) cursor: Vec2,
    pub(crate) screen: Vec2,
    pub(crate) hovering: f32,
    pub(crate) screen_scale: f32,
}

#[derive(Clone, Copy, Debug, ShaderType)]
struct FaceParams {
    rect: Vec4,
    tint: Vec4,
}

/// One atlas cell of a card sprite, drawn with the tilt vertex stage.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub(crate) struct TiltImage {
    #[uniform(0)]
    face: FaceParams,
    #[texture(1)]
    #[sampler(2)]
    image: Handle<Image>,
    #[uniform(3)]
    tilt: TiltParams,
}

impl UiMaterial for TiltImage {
    fn vertex_shader() -> ShaderRef {
        TILT_SHADER.into()
    }

    fn fragment_shader() -> ShaderRef {
        "embedded://ante_app/components/tilt_image.wgsl".into()
    }
}

/// The card whose hover drives this node's tilt.
#[derive(Component)]
pub(crate) struct TiltRoot(Entity);

/// A sprite or overlay outside any card, left as it is.
#[derive(Component)]
pub(crate) struct NoTilt;

/// The nearest ancestor (or the node itself) that is a hoverable card.
fn card_root(
    entity: Entity,
    parents: &Query<&ChildOf>,
    cards: &Query<(), With<HoverMotion>>,
) -> Option<Entity> {
    let mut current = entity;
    for _ in 0..8 {
        if cards.contains(current) {
            return Some(current);
        }
        current = parents.get(current).ok()?.parent();
    }
    None
}

/// The atlas cell of `image` in normalised coordinates, once its assets are loaded.
fn cell(
    image: &ImageNode,
    layouts: &Assets<TextureAtlasLayout>,
    images: &Assets<Image>,
) -> Option<Vec4> {
    if let Some(atlas) = &image.texture_atlas {
        let layout = layouts.get(&atlas.layout)?;
        let rect = layout.textures.get(atlas.index)?.as_rect();
        let size = layout.size.as_vec2();
        return Some(Vec4::new(
            rect.min.x / size.x,
            rect.min.y / size.y,
            rect.width() / size.x,
            rect.height() / size.y,
        ));
    }
    if let Some(rect) = image.rect {
        let size = images.get(&image.image)?.size_f32();
        return Some(Vec4::new(
            rect.min.x / size.x,
            rect.min.y / size.y,
            rect.width() / size.x,
            rect.height() / size.y,
        ));
    }
    images
        .get(&image.image)
        .map(|_| Vec4::new(0.0, 0.0, 1.0, 1.0))
}

#[allow(clippy::type_complexity)]
fn tilt_card_sprites(
    mut commands: Commands,
    sprites: Query<(Entity, &ImageNode), (Without<NoTilt>, Without<TiltRoot>)>,
    parents: Query<&ChildOf>,
    cards: Query<(), With<HoverMotion>>,
    layouts: Res<Assets<TextureAtlasLayout>>,
    images: Res<Assets<Image>>,
    mut materials: ResMut<Assets<TiltImage>>,
) {
    for (entity, image) in &sprites {
        let plain =
            !image.flip_x && !image.flip_y && matches!(image.image_mode, NodeImageMode::Auto);
        let Some(root) = plain.then(|| card_root(entity, &parents, &cards)).flatten() else {
            commands.entity(entity).insert(NoTilt);
            continue;
        };
        // Retried next frame until the atlas and image are loaded.
        let Some(rect) = cell(image, &layouts, &images) else {
            continue;
        };
        let material = materials.add(TiltImage {
            face: FaceParams {
                rect,
                tint: image.color.to_linear().to_vec4(),
            },
            image: image.image.clone(),
            tilt: TiltParams::default(),
        });
        commands
            .entity(entity)
            .remove::<ImageNode>()
            .insert((MaterialNode(material), TiltRoot(root)));
    }
}

#[allow(clippy::type_complexity)]
fn root_overlays(
    mut commands: Commands,
    overlays: Query<
        Entity,
        (
            With<MaterialNode<LoveMaterial>>,
            Without<NoTilt>,
            Without<TiltRoot>,
        ),
    >,
    parents: Query<&ChildOf>,
    cards: Query<(), With<HoverMotion>>,
) {
    for entity in &overlays {
        match card_root(entity, &parents, &cards) {
            Some(root) => commands.entity(entity).insert(TiltRoot(root)),
            None => commands.entity(entity).insert(NoTilt),
        };
    }
}

/// The tilt uniforms of a card: its hover and the cursor, or nothing when not hovered.
pub(crate) fn tilt_params(
    hovered: bool,
    cursor: Option<Vec2>,
    screen: Vec2,
    pixels_per_unit: f32,
) -> TiltParams {
    match cursor.filter(|_| hovered) {
        // `Card:hover` sets `hover_tilt = 1`.
        Some(cursor) => TiltParams {
            cursor,
            screen,
            hovering: 1.0,
            screen_scale: pixels_per_unit,
        },
        None => TiltParams::default(),
    }
}

#[allow(clippy::type_complexity)]
fn update_tilt(
    windows: Query<&Window, With<PrimaryWindow>>,
    cards: Query<(&HoverMotion, &UiGlobalTransform, &UiTransform)>,
    nodes: Query<(
        &TiltRoot,
        Option<&MaterialNode<TiltImage>>,
        Option<&MaterialNode<LoveMaterial>>,
    )>,
    mut faces: ResMut<Assets<TiltImage>>,
    mut overlays: ResMut<Assets<LoveMaterial>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let cursor = window.physical_cursor_position();
    let screen = Vec2::new(
        window.physical_width() as f32,
        window.physical_height() as f32,
    );
    for (root, face, overlay) in &nodes {
        let Ok((motion, global, transform)) = cards.get(root.0) else {
            continue;
        };
        // `screen_scale`: physical pixels per room unit, the room fit included.
        let room = global.matrix2.x_axis.length() / transform.scale.x.max(1e-6);
        let params = tilt_params(
            motion.hovered(),
            cursor,
            screen,
            TILE * room * window.scale_factor(),
        );
        if let Some(face) = face
            && faces
                .get(&face.0)
                .is_some_and(|material| material.tilt != params)
            && let Some(mut material) = faces.get_mut(&face.0)
        {
            material.tilt = params;
        }
        if let Some(overlay) = overlay
            && overlays
                .get(&overlay.0)
                .is_some_and(|material| material.tilt() != params)
            && let Some(mut material) = overlays.get_mut(&overlay.0)
        {
            material.set_tilt(params);
        }
    }
}

#[cfg(test)]
mod tests;

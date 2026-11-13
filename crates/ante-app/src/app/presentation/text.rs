use super::*;

/// Size rounded to whole physical pixels, so text does not shimmer as the window scales.
/// Sizes are not snapped to the font's glyph grid: at 1080p that step is about 16 logical
/// pixels and would shrink or grow small text by up to a third.
pub(crate) fn snap_font_size(size: f32, scale: f32) -> f32 {
    let scale = scale.max(0.01);
    (size * scale).round().max(1.0) / scale
}

/// The size a text was asked for, before pixel snapping.
#[derive(Component, Clone, Copy)]
pub(crate) struct AskedFontSize(f32);

/// A text size changed at runtime, in pixels before snapping. Changing this instead of
/// reinserting `TextFont` keeps the game font.
#[derive(Component, Clone, Copy, PartialEq)]
pub(crate) struct TextSize(pub(crate) f32);

pub(crate) fn apply_text_size(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut fonts: Query<(&TextSize, &mut TextFont, Entity), Changed<TextSize>>,
    mut commands: Commands,
) {
    let scale = windows.single().map_or(1.0, |window| window.scale_factor());
    for (size, mut text, entity) in &mut fonts {
        commands.entity(entity).insert(AskedFontSize(size.0));
        text.font_size = bevy::text::FontSize::Px(snap_font_size(size.0, scale));
    }
}

pub(crate) fn apply_local_font(
    atlases: Res<Atlases>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut fonts: Query<(&mut TextFont, Option<&AskedFontSize>, Entity), Changed<TextFont>>,
    mut commands: Commands,
) {
    let scale = windows.single().map_or(1.0, |window| window.scale_factor());
    for (mut text, asked, entity) in &mut fonts {
        // Also catches a `TextFont` reinserted with the default font.
        if let Some(font) = &atlases.font
            && !matches!(&text.font, bevy::text::FontSource::Handle(handle) if handle == font)
        {
            text.font = bevy::text::FontSource::Handle(font.clone());
        }
        if asked.is_none()
            && let bevy::text::FontSize::Px(size) = text.font_size
        {
            commands.entity(entity).insert(AskedFontSize(size));
            text.font_size = bevy::text::FontSize::Px(snap_font_size(size, scale));
        }
    }
}

/// Re-snaps every text when the window scale changes (resize).
pub(crate) fn resnap_fonts(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut last: Local<f32>,
    mut fonts: Query<(&mut TextFont, &AskedFontSize)>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let scale = window.scale_factor();
    if (scale - *last).abs() < 1e-4 {
        return;
    }
    *last = scale;
    for (mut text, asked) in &mut fonts {
        text.font_size = bevy::text::FontSize::Px(snap_font_size(asked.0, scale));
    }
}

pub(crate) fn use_nearest_glyph_sampling(
    atlases: Res<FontAtlasSet>,
    mut images: ResMut<Assets<Image>>,
) {
    let nearest = ImageSampler::nearest();
    for atlas in atlases.values().flatten() {
        let id = atlas.texture.id();
        if images.get(id).is_some_and(|image| image.sampler != nearest)
            && let Some(mut image) = images.get_mut(id)
        {
            image.sampler = nearest.clone();
        }
    }
}

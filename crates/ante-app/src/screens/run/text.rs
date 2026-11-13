use super::*;

/// DynaText `float`: a slow sway and bob.
#[derive(Component)]
pub(crate) struct Floating;

/// Text read from the local localization by key.
#[derive(Component)]
pub(crate) struct LocalText {
    pub(super) key: &'static str,
    pub(super) var: Option<String>,
}

impl LocalText {
    pub(crate) fn new(key: &'static str) -> Self {
        Self { key, var: None }
    }

    /// A string with `#1#` replaced by `var`.
    pub(crate) fn with_var(key: &'static str, var: impl ToString) -> Self {
        Self {
            key,
            var: Some(var.to_string()),
        }
    }
}

pub(crate) fn float_text(
    time: Res<Time<Real>>,
    mut texts: Query<&mut UiTransform, With<Floating>>,
) {
    let t = time.elapsed_secs();
    for mut transform in &mut texts {
        transform.rotation = Rot2::radians(0.02 * (2.0 * t).sin());
        transform.translation = Val2::px(0.0, -(1.0 + 0.3 * (0.666 * t).sin()) * 0.05 * TILE);
    }
}

pub(crate) fn fill_local_text(
    text: Option<Res<crate::services::assets::ui_text::UiText>>,
    mut nodes: Query<(&LocalText, &mut Text), Added<LocalText>>,
) {
    let Some(text) = text else {
        return;
    };
    for (local, mut value) in &mut nodes {
        value.0 = match &local.var {
            Some(var) => text.with_var(local.key, var),
            None => text.get(local.key),
        };
    }
}

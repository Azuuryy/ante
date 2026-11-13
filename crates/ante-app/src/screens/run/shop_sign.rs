use super::*;

/// `G.SHOP_SIGN`: the animated sign and the gold line in a DYN_UI.DARK box framed by MAIN.
pub(super) fn spawn_shop_sign(ui: &mut ChildSpawnerCommands, image: ImageNode) -> Entity {
    ui.spawn((
        shop_sign_node(Display::None),
        PixelBox::new(BLACK).emboss(0.05),
        DynRole::Main,
        Pickable::IGNORE,
    ))
    .with_children(|frame| {
        frame
            .spawn((
                Node {
                    min_width: px(4.72 * TILE),
                    min_height: px(3.1 * TILE),
                    padding: UiRect::all(px(0.1 * TILE)),
                    justify_content: JustifyContent::Center,
                    ..column(0.1 * TILE)
                },
                PixelBox::new(BLACK),
                DynRole::Dark,
                Pickable::IGNORE,
            ))
            .with_children(|sign| {
                sign.spawn((
                    image,
                    Node {
                        width: px(4.4 * TILE),
                        height: px(2.2 * TILE),
                        ..default()
                    },
                    ShopSignFrames,
                    Pickable::IGNORE,
                ));
                sign.spawn((
                    Text::new(""),
                    TextFont {
                        font_size: bevy::text::FontSize::Px(text_px(0.5)),
                        ..default()
                    },
                    TextColor(lighten(crate::components::tiles::GOLD, 0.3)),
                    text_shadow(),
                    LocalText::new("ph_improve_run"),
                    Floating,
                    UiTransform::IDENTITY,
                    Pickable::IGNORE,
                ));
            });
    })
    .id()
}

pub(super) fn shop_sign_node(display: Display) -> Node {
    Node {
        display,
        padding: UiRect::all(px(0.1 * TILE)),
        ..default()
    }
}

/// The shop sign atlas holds 4 frames of 113 x 57, played at `G.ANIMATION_FPS` = 10.
#[derive(Component)]
pub(crate) struct ShopSignFrames;

pub(super) const SIGN_FRAMES: u32 = 4;

pub(super) const SIGN_FRAME: Vec2 = Vec2::new(113.0, 57.0);

pub(super) const ANIMATION_FPS: f32 = 10.0;

pub(crate) fn animate_shop_sign(
    time: Res<Time<Real>>,
    mut signs: Query<&mut ImageNode, With<ShopSignFrames>>,
) {
    let frame = (time.elapsed_secs() * ANIMATION_FPS) as u32 % SIGN_FRAMES;
    let min = Vec2::new(frame as f32 * SIGN_FRAME.x, 0.0);
    for mut image in &mut signs {
        let rect = Rect::from_corners(min, min + SIGN_FRAME);
        if image.rect != Some(rect) {
            image.rect = Some(rect);
        }
    }
}

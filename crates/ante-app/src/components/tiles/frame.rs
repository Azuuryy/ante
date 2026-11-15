use super::*;

/// `G.C.JOKER_GREY`.
pub const JOKER_GREY: Color = Color::srgb(
    0xBF as f32 / 255.0,
    0xC7 as f32 / 255.0,
    0xD5 as f32 / 255.0,
);

/// `create_UIBox_generic_options`: a JOKER_GREY frame (padding 0.07, emboss 0.1) around an
/// L_BLACK body (padding 0.2) holding `content`, then the orange Back button across the
/// body (`minw = 2.5`, padding 0.1, "Back" at scale 0.5).
pub fn generic_options(
    ui: &mut ChildSpawnerCommands,
    back: Click,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    ui.spawn((
        Node {
            padding: UiRect::all(tiles(0.07)),
            flex_shrink: 0.0,
            ..default()
        },
        PixelBox::new(JOKER_GREY).emboss(0.1),
    ))
    .with_children(|frame| {
        frame
            .spawn((
                Node {
                    padding: UiRect::all(tiles(0.2)),
                    flex_direction: FlexDirection::Column,
                    row_gap: tiles(0.2),
                    ..default()
                },
                PixelBox::new(L_BLACK),
            ))
            .with_children(|body| {
                body.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: tiles(0.1),
                    ..default()
                })
                .with_children(content);
                let mut button = body.spawn((
                    Node {
                        min_width: tiles(2.5),
                        padding: UiRect::all(tiles(0.1)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    PixelBox::new(ORANGE).shadow(),
                ));
                attach_button(&mut button, ORANGE, back);
                button.with_children(|label| {
                    shadowed_label(label, "Back", text_px(0.5), INK);
                });
            });
    });
}

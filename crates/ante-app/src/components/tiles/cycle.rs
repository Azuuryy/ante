use super::*;

/// `create_option_cycle` with `cycle_shoulders` and `no_pips`: red "<" and ">" buttons
/// (0.6 wide, text 0.5) on each side of a red box embossed 0.1, `w` × 0.8, showing the
/// current option. With fewer than two options the arrows turn black and inactive.
pub fn option_cycle(
    ui: &mut ChildSpawnerCommands,
    text: &str,
    w: f32,
    previous: Option<Click>,
    next: Option<Click>,
) {
    ui.spawn(Node {
        padding: UiRect::horizontal(tiles(0.7)),
        ..default()
    })
    .with_children(|shoulders| {
        shoulders
            .spawn(Node {
                padding: UiRect::all(tiles(0.1)),
                column_gap: tiles(0.1),
                align_items: AlignItems::Stretch,
                ..default()
            })
            .with_children(|cycle| {
                cycle_arrow(cycle, "<", previous);
                cycle
                    .spawn((
                        Node {
                            min_width: tiles(w),
                            min_height: tiles(0.8),
                            padding: UiRect::all(tiles(0.05)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        PixelBox::new(RED).emboss(0.1),
                    ))
                    .with_children(|value| {
                        shadowed_label(value, text, text_px(0.5), INK);
                    });
                cycle_arrow(cycle, ">", next);
            });
    });
}

pub fn cycle_arrow(ui: &mut ChildSpawnerCommands, arrow: &str, action: Option<Click>) {
    let color = if action.is_some() { RED } else { BLACK };
    let mut node = ui.spawn((
        Node {
            min_width: tiles(0.6),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        if action.is_some() {
            PixelBox::new(color).shadow()
        } else {
            PixelBox::new(color)
        },
    ));
    if let Some(action) = action {
        attach_button(&mut node, color, action);
    }
    node.with_children(|label| {
        shadowed_label(
            label,
            arrow,
            text_px(0.5),
            if action.is_some() { INK } else { TEXT_INACTIVE },
        );
    });
}

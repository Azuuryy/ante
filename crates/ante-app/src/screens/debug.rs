use ante_core::contract::{Command as MvpAction, DebugCommand, Phase, RunView};
use bevy::prelude::*;

use crate::{
    app::session::Click,
    components::{
        theme::INK,
        tiles::{BLUE, ORANGE, RED, tile_optional},
        widgets::label,
    },
};

/// Dev shortcuts, opened with Tab like the original debug tools panel.
pub(crate) fn debug_panel(ui: &mut ChildSpawnerCommands, view: &RunView) {
    let playing = view.phase == Phase::Playing;
    let running = !matches!(view.phase, Phase::Won | Phase::Lost);
    let entries = [
        ("+$10", DebugCommand::AddMoney(10), ORANGE, running),
        ("+$100", DebugCommand::AddMoney(100), ORANGE, running),
        ("+1 Hand", DebugCommand::AddHand, BLUE, playing),
        ("+1 Discard", DebugCommand::AddDiscard, RED, playing),
        ("+1 Round", DebugCommand::AddRound, BLUE, running),
        ("+1 Ante", DebugCommand::AddAnte, BLUE, running && !playing),
        ("+100 Score", DebugCommand::AddScore(100), BLUE, playing),
        ("Win Blind", DebugCommand::WinBlind, ORANGE, playing),
        ("Lose Run", DebugCommand::LoseRun, RED, running),
    ];
    ui.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: px(12),
            top: px(230),
            padding: UiRect::all(px(10)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(6),
            border_radius: BorderRadius::all(px(10)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.86, 0.88, 0.90, 0.95)),
        GlobalZIndex(150),
    ))
    .with_children(|panel| {
        label(panel, "Debug (Tab)", 18.0, INK);
        for (text, command, color, enabled) in entries {
            tile_optional(
                panel,
                text,
                color,
                (2.6, 0.5),
                0.4,
                enabled.then_some(Click::Play(MvpAction::Debug(command))),
                false,
            );
        }
        if view.debug {
            label(panel, "Debug run", 14.0, RED);
        }
    });
}

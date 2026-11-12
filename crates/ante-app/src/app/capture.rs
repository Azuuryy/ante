use std::time::{SystemTime, UNIX_EPOCH};

use ante_core::contract::Phase;
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
    window::PrimaryWindow,
};
use serde_json::json;

use crate::{
    app::{navigation::Screen, session::Session},
    services::{diagnostics::journal, storage},
};

/// F6 saves the window for side-by-side comparison with reference captures.
pub(crate) fn capture_window(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    session: Res<Session>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if !keys.just_pressed(KeyCode::F6) {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let surface = surface(
        session.navigation.screen,
        session.confirmed.as_ref().map(|view| view.phase),
    );
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis())
        .unwrap_or_default();
    let name = file_name(
        surface,
        stamp,
        window.physical_width(),
        window.physical_height(),
    );
    let directory = storage::local_dir().join("captures");
    if let Err(error) = std::fs::create_dir_all(&directory) {
        journal::event("capture_failed", json!({"error": error.to_string()}));
        return;
    }
    let path = directory.join(&name);
    journal::event("capture_saved", json!({"file": name}));
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
}

/// Tab toggles the debug panel during a run, like the original debug tools.
pub(crate) fn toggle_debug_panel(keys: Res<ButtonInput<KeyCode>>, mut session: ResMut<Session>) {
    if keys.just_pressed(KeyCode::Tab) && session.navigation.screen == Screen::Run {
        session.debug_panel = !session.debug_panel;
    }
}

fn surface(screen: Screen, phase: Option<Phase>) -> &'static str {
    if screen != Screen::Run {
        return match screen {
            Screen::Main => "menu",
            Screen::DeckSelect => "run-setup",
            Screen::Options => "options",
            Screen::Pause => "pause",
            Screen::RunInfo => "run-info",
            Screen::DeckInspect => "deck-inspect",
            Screen::Collection => "collection",
            Screen::Run => "run",
        };
    }
    match phase {
        Some(Phase::BeforeBlind) => "blind-select",
        Some(Phase::Reward) => "cash-out",
        Some(Phase::Shop) => "shop",
        Some(Phase::Pack) => "shop_pack-open",
        Some(Phase::Won) => "end_win",
        Some(Phase::Lost) => "end_lose",
        Some(Phase::Playing | Phase::Target) | None => "run",
    }
}

fn file_name(surface: &str, stamp: u128, width: u32, height: u32) -> String {
    format!("{surface}_{stamp}_{width}x{height}.png")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_names_follow_the_library_convention() {
        assert_eq!(
            file_name(surface(Screen::Run, Some(Phase::Shop)), 42, 1920, 1080),
            "shop_42_1920x1080.png"
        );
        assert_eq!(surface(Screen::Main, None), "menu");
    }
}

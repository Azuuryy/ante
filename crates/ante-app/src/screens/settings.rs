use crate::{
    app::session::{Click, Session},
    components::{
        theme::{INK, MUTED},
        tiles as menu_ui,
        widgets::{button, label, row},
    },
    services::{diagnostics::FramePacing, profile::Profile},
};
use bevy::prelude::*;

pub(crate) fn show_options(
    ui: &mut ChildSpawnerCommands,
    session: &Session,
    pacing: &FramePacing,
    profile: &Profile,
) {
    use menu_ui::{BLUE, ORANGE, RED, tile_button as tile};
    menu_ui::screen_heading(ui, "SETTINGS");
    menu_ui::panel(ui, |panel| {
        // One row of four tabs: wider tiles wrapped AUDIO onto the next line.
        panel
            .spawn(Node {
                column_gap: menu_ui::tiles(0.12),
                ..default()
            })
            .with_children(|tabs| {
                for (index, name) in ["GAME", "VIDEO", "GRAPHICS", "AUDIO"].iter().enumerate() {
                    menu_ui::tile_optional(
                        tabs,
                        name,
                        RED,
                        (2.05, 0.75),
                        0.36,
                        Some(Click::OptionsTab(index as u8)),
                        session.options_tab == index as u8,
                    );
                }
            });
        match session.options_tab {
            0 => {
                label(panel, "Score animation speed", menu_ui::text_px(0.42), INK);
                tile(
                    panel,
                    &format!("<     {}     >", session.speed.label()),
                    BLUE,
                    (4.6, 0.8),
                    0.43,
                    Click::ChangeSpeed,
                );
                #[cfg(feature = "dev-tools")]
                {
                    label(panel, "Debug profile", menu_ui::text_px(0.36), MUTED);
                    if profile.all_unlocked() {
                        label(
                            panel,
                            "Collection fully unlocked",
                            menu_ui::text_px(0.34),
                            MUTED,
                        );
                    }
                    button(
                        panel,
                        if profile.all_unlocked() {
                            "Unlock all again (debug)"
                        } else {
                            "Unlock all (debug)"
                        },
                        // Stays available so categories added later can be unlocked too.
                        profile.is_writable().then_some(Click::UnlockAll),
                    );
                }
            }
            1 => {
                label(panel, "Vertical sync", menu_ui::text_px(0.42), INK);
                tile(
                    panel,
                    &format!("<   {}   >", pacing.label()),
                    BLUE,
                    (7.0, 0.8),
                    0.30,
                    Click::ToggleVsync,
                );
            }
            2 => {
                let assets = &session.asset_settings;
                label(panel, "Local Assets", 26.0, INK);
                label(panel, format!("Source: {}", assets.source), 17.0, INK);
                label(
                    panel,
                    format!("Destination: {}", assets.destination().display()),
                    16.0,
                    MUTED,
                );
                label(panel, &assets.status, 18.0, INK);
                row(panel, |actions| {
                    button(
                        actions,
                        if assets.editing {
                            "Editing path"
                        } else {
                            "Edit path"
                        },
                        Some(Click::AssetEdit),
                    );
                    button(actions, "Paste", Some(Click::AssetPaste));
                    button(actions, "Steam", Some(Click::AssetSteam));
                    button(actions, "Import", Some(Click::AssetImport));
                });
                row(panel, |actions| {
                    if assets.confirm_remove {
                        button(actions, "Confirm remove", Some(Click::AssetRemove));
                        button(actions, "Cancel", Some(Click::AssetCancelRemove));
                    } else {
                        button(
                            actions,
                            "Remove local copy",
                            assets.active.is_some().then_some(Click::AssetRemove),
                        );
                    }
                });
            }
            _ => {
                // `G.UIDEF.settings_tab('Audio')`: master, music and game sound sliders.
                use crate::services::profile::VolumeChannel::{Game, Master, Music};
                for (title, channel) in [
                    ("Master Volume", Master),
                    ("Music Volume", Music),
                    ("Game Volume", Game),
                ] {
                    menu_ui::volume_slider(panel, title, channel, profile.volume_of(channel));
                }
                label(
                    panel,
                    "Sounds play from your local installation",
                    menu_ui::text_px(0.3),
                    MUTED,
                );
            }
        }
        tile(panel, "BACK", ORANGE, (2.8, 0.8), 0.45, Click::Back);
    });
}

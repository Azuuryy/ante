use ante_core::contract::{CardKey, JokerKey, Phase, ScoreSource};
use bevy::{image::ImageSampler, prelude::*, text::FontAtlasSet, window::PrimaryWindow};

use crate::{
    app::{
        navigation::{DeckChoice, Screen as Page},
        playback::{Popup, Stage as PlaybackStage},
        session::{Backend, Click, Session},
    },
    components::{
        room::RoomShake,
        theme::{FELT, INK},
        tiles as menu_ui,
        travel::{DRAW_DELAY, Travel},
        widgets::{HIGHLIGHT_LIFT, HoverMotion, PlayedCard, button, label},
    },
    screens::{
        collection::show_collection, menu::main_menu, run as run_ui, settings::show_options,
    },
    services::{
        assets::{
            catalog::Atlases,
            sounds::{ScheduledSfx, Sfx},
        },
        diagnostics::{self, FrameMetrics, FramePacing},
        profile::Profile,
    },
};

mod notes;
mod popup;
mod score;
mod text;
mod unlock_toast;

pub(crate) use notes::*;
pub(crate) use popup::*;
pub(crate) use score::*;
pub(crate) use text::*;
pub(crate) use unlock_toast::*;
#[derive(Component)]
pub(crate) struct Screen;

#[derive(Component)]
pub(crate) struct PlaybackLabel;

/// The resources `render` reads besides the session.
type RenderSources<'w> = (
    Res<'w, Profile>,
    Res<'w, Backend>,
    Option<Res<'w, crate::services::assets::ui_text::UiText>>,
    Option<Res<'w, crate::services::assets::shaders::LoveShaders>>,
);

pub(crate) fn render(
    mut commands: Commands,
    session: Res<Session>,
    screens: Query<Entity, With<Screen>>,
    atlases: Res<Atlases>,
    (profile, backend, ui_text, shaders): RenderSources,
    mut metrics: ResMut<FrameMetrics>,
    (pacing, mut previous_mode, mut surface): (
        Res<FramePacing>,
        Local<Option<bevy::window::PresentMode>>,
        Local<Option<run_ui::RunBoard>>,
    ),
) {
    let mode = pacing.present_mode();
    if !session.is_changed() && *previous_mode == Some(mode) {
        return;
    }
    *previous_mode = Some(mode);
    let start = diagnostics::measurement();
    let page = session.navigation.screen;
    let board_visible =
        !session.faulted && (page == Page::Run || session.navigation.overlays_run());
    if board_visible && let Some(current) = &session.confirmed {
        let deck = session
            .last_config
            .and_then(|config| {
                DeckChoice::ALL
                    .into_iter()
                    .find(|choice| choice.id() == config.deck.id())
            })
            .unwrap_or(DeckChoice::Red);
        if surface
            .as_ref()
            .is_none_or(|board| board.run_id != session.run_id || screens.get(board.root).is_err())
        {
            for entity in &screens {
                commands.entity(entity).despawn();
            }
            *surface = Some(run_ui::RunBoard::spawn(
                &mut commands,
                &atlases,
                deck,
                session.run_id,
                session.stake,
            ));
        }
        let board = surface.as_mut().unwrap();
        let view = if let Some((before, _)) = &session.playback {
            before
        } else {
            current
        };
        let fallback_text = crate::services::assets::ui_text::UiText::default();
        let text = ui_text.as_deref().unwrap_or(&fallback_text);
        board.update(&mut commands, view, &atlases, &session, text);
        commands
            .entity(board.overlay)
            .despawn_children()
            .with_children(|ui| {
                #[cfg(feature = "dev-tools")]
                if page == Page::Run && session.debug_panel {
                    crate::screens::debug::debug_panel(ui, view);
                }
                if page == Page::Run && matches!(view.phase, Phase::Won | Phase::Lost) {
                    run_ui::overlay(ui, |modal| run_ui::result(modal, view));
                } else if page != Page::Run {
                    run_ui::overlay(ui, |modal| match page {
                        Page::DeckSelect => {
                            run_ui::deck_select(modal, &session, &atlases, &*backend.0, &profile)
                        }
                        Page::Pause => run_ui::pause(modal, session.playback.is_some()),
                        Page::RunInfo => {
                            run_ui::run_info(modal, view, &atlases, session.run_info_tab)
                        }
                        Page::DeckInspect => run_ui::inspect_deck(
                            modal,
                            view,
                            &atlases,
                            deck,
                            session.deck_remaining,
                        ),
                        Page::Options => show_options(modal, &session, &pacing, &profile),
                        Page::Collection => {
                            show_collection(modal, &session, &profile, &atlases, &*backend.0)
                        }
                        Page::Main | Page::Run => {}
                    });
                }
            });
    } else {
        *surface = None;
        for entity in &screens {
            commands.entity(entity).despawn();
        }
        let root = commands
            .spawn((
                Screen,
                Node {
                    width: px(menu_ui::CANVAS.x),
                    height: px(menu_ui::CANVAS.y),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(FELT),
                UiTransform::IDENTITY,
            ))
            .id();
        // `G.SPLASH_BACK` covers the whole canvas behind the room.
        let splash = shaders
            .as_ref()
            .is_some_and(|shaders| shaders.get("splash").is_some());
        if splash {
            commands
                .entity(root)
                .with_child(crate::components::background::splash_layer());
        }
        let room = commands.spawn(room_node()).id();
        commands.entity(root).add_child(room);
        commands.entity(room).with_children(|ui| {
            if session.faulted {
                run_ui::overlay(ui, |modal| {
                    label(modal, "Run stopped after a simulation error", 32.0, INK);
                    if let Some(error) = &session.error {
                        label(modal, error, 20.0, INK);
                    }
                    button(modal, "Seed menu", Some(Click::Menu));
                });
                return;
            }
            main_menu(ui, &atlases, splash);
            if page != Page::Main {
                run_ui::overlay(ui, |modal| match page {
                    Page::DeckSelect => {
                        run_ui::deck_select(modal, &session, &atlases, &*backend.0, &profile)
                    }
                    Page::Options => show_options(modal, &session, &pacing, &profile),
                    Page::Collection => {
                        show_collection(modal, &session, &profile, &atlases, &*backend.0)
                    }
                    _ => {}
                });
            }
        });
    }
    metrics.record_ui(start.elapsed());
}

/// The 20 x 11.5 tile room, centred in the canvas inside the original room padding.
pub(crate) fn room_node() -> Node {
    Node {
        width: px(menu_ui::ROOM.x),
        height: px(menu_ui::ROOM.y),
        flex_shrink: 0.0,
        ..default()
    }
}

pub(crate) fn fit_room(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut roots: Query<&mut Node, With<Screen>>,
) {
    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    let (canvas, scale) = menu_ui::viewport_fit(window.resolution.physical_size().as_vec2());
    if window.resolution.scale_factor_override() != Some(scale) {
        window.resolution.set_scale_factor_override(Some(scale));
    }
    for mut node in &mut roots {
        if node.width != px(canvas.x) || node.height != px(canvas.y) {
            node.width = px(canvas.x);
            node.height = px(canvas.y);
        }
    }
}

#[cfg(test)]
mod tests;

mod app;
mod components;
mod screens;
mod services;

pub use app::playback;

use std::path::PathBuf;

use ante_core::contract::{CampaignGoal, GameBackend};
use bevy::{
    log::LogPlugin, prelude::*, render::error_handler::RenderErrorHandler, ui::IsDefaultUiCamera,
    window::ExitCondition,
};

#[cfg(feature = "dev-tools")]
use crate::services::diagnostics::journal;
use crate::{
    app::{
        presentation::{
            animate, apply_local_font, apply_text_size, fit_room, render, resnap_fonts,
            use_nearest_glyph_sampling,
        },
        session::{Backend, Click, Session, edit_seed, process_clicks, right_click_clear},
    },
    components::{card_drag, tooltip},
    services::{
        assets::{
            catalog::{self as atlas, AtlasSource, Atlases},
            import as local_assets,
        },
        diagnostics::{FramePacing, PerformancePlugin},
        profile::Profile,
    },
};

macro_rules! journal_event {
    ($($tokens:tt)*) => {{
        #[cfg(feature = "dev-tools")]
        crate::services::diagnostics::journal::event($($tokens)*);
    }};
}
pub(crate) use journal_event;

pub fn run(backend: impl GameBackend) {
    #[cfg(feature = "dev-tools")]
    journal::init();
    let mut args = std::env::args_os().skip(1);
    let mut source = None;
    let mut benchmark = false;
    let mut goal = CampaignGoal::Standard;
    while let Some(arg) = args.next() {
        if arg == "--assets" {
            source = args.next().map(PathBuf::from);
        } else if arg == "--steam-textures" {
            source = local_assets::steam_source();
        } else if arg == "--one-ante" {
            goal = CampaignGoal::SingleAnte;
        } else if cfg!(feature = "dev-tools") && arg == "--benchmark-fps" {
            benchmark = true;
        }
    }
    let asset_settings = local_assets::AssetSettings::load();
    let source = source.or_else(|| asset_settings.active.clone());
    let pacing = FramePacing::new(benchmark);
    let present_mode = pacing.present_mode();
    let defaults = DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: if cfg!(feature = "dev-tools") {
                "Ante Dev".into()
            } else {
                "Ante".into()
            },
            resolution: (1080, 700).into(),
            present_mode,
            ..default()
        }),
        exit_condition: ExitCondition::OnPrimaryClosed,
        ..default()
    });
    #[cfg(feature = "dev-tools")]
    let defaults = defaults.set(LogPlugin {
        custom_layer: journal::bevy_layer,
        ..default()
    });
    #[cfg(not(feature = "dev-tools"))]
    let defaults = defaults.disable::<LogPlugin>();
    let profile = Profile::load();
    let mut session = Session::new(goal, asset_settings);
    if let Some(speed) = profile
        .game_speed()
        .and_then(app::playback::Speed::from_factor)
    {
        session.speed = speed;
    }
    let mut app = App::new();
    #[cfg(feature = "dev-tools")]
    app.set_error_handler(journal::ecs_error).add_systems(
        Update,
        (
            app::capture::capture_window,
            app::capture::toggle_debug_panel,
        ),
    );
    app.insert_resource(Backend(Box::new(backend)))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(pacing)
        .insert_resource(session)
        .insert_resource(AtlasSource(source))
        .insert_resource(profile)
        .init_resource::<Atlases>()
        .init_resource::<tooltip::TooltipState>()
        .init_resource::<card_drag::CardDrag>()
        .init_resource::<components::tiles::SliderGrab>()
        .init_resource::<screens::run::RoundEvalClock>()
        .init_resource::<components::travel::CardTrail>()
        .add_message::<Click>()
        .add_message::<services::assets::sounds::Sfx>()
        .init_resource::<services::assets::sounds::ScheduledSfx>()
        .init_resource::<components::room::RoomShake>()
        .add_plugins(defaults)
        .insert_resource(RenderErrorHandler(render_error))
        .add_plugins((
            PerformancePlugin,
            components::edition::EditionPlugin,
            components::pixel_rect::PixelRectPlugin,
            components::tilt::TiltPlugin,
            components::background::BackgroundPlugin,
        ))
        .add_systems(
            Update,
            (
                components::travel::retire_discards,
                components::room::shake_room,
                components::dissolve::dissolve,
                components::dissolve::move_particles,
                components::widgets::hide_debuff_fallback,
                (
                    app::cues::pack_open_sounds,
                    services::assets::sounds::release_scheduled,
                    services::assets::sounds::play,
                )
                    .chain(),
                services::assets::music::modulate_music,
            ),
        )
        .add_systems(
            Startup,
            (
                |mut commands: Commands| {
                    commands.spawn((Camera2d, IsDefaultUiCamera));
                },
                atlas::load,
                services::assets::shaders::load,
                services::assets::sounds::load,
                services::assets::music::load,
                services::assets::ui_text::load,
            ),
        )
        .add_systems(
            Update,
            (
                screens::run::animate_shop_sign,
                screens::run::float_text,
                screens::run::fill_local_text,
                screens::run::reveal_round_eval,
                screens::hand::paint_hand_actions,
                components::tiles::drag_sliders,
            ),
        )
        .add_systems(
            Update,
            (
                edit_seed,
                right_click_clear,
                card_drag::update,
                app::cues::click_sounds,
                process_clicks,
                components::travel::detach_discards,
                components::dissolve::detach_leaving,
                animate,
                crate::components::widgets::fan_hand,
                crate::components::widgets::ease_hover,
                components::travel::turn_card_backs,
                render,
                components::selection::apply_selection,
                fit_room,
                tooltip::update,
                tooltip::animate_edition_badges,
                tooltip::animate_shimmer,
                apply_local_font,
                resnap_fonts,
            )
                .chain(),
        )
        .add_systems(Update, apply_text_size.after(apply_local_font))
        .init_resource::<components::sparkles::PackOpening>()
        .add_systems(Update, components::sparkles::update.after(render))
        .add_systems(
            PostUpdate,
            (
                use_nearest_glyph_sampling
                    .after(bevy::ui::UiSystems::PostLayout)
                    .before(bevy::asset::AssetEventSystems),
                app::presentation::snap_glyphs.after(bevy::ui::UiSystems::PostLayout),
                components::travel::prepare.before(bevy::ui::UiSystems::Layout),
                components::travel::place
                    .after(bevy::ui::UiSystems::Layout)
                    .before(bevy::ui::UiSystems::PostLayout)
                    .before(bevy::camera::visibility::VisibilitySystems::VisibilityPropagate),
            ),
        )
        .run();
}

fn render_error(
    error: &bevy::render::error_handler::RenderError,
    world: &mut World,
    render_world: &mut World,
) -> bevy::render::error_handler::RenderErrorPolicy {
    #[cfg(feature = "dev-tools")]
    {
        journal::render_error(error, world, render_world)
    }
    #[cfg(not(feature = "dev-tools"))]
    {
        let _ = (error, render_world);
        world.write_message(AppExit::error());
        bevy::render::error_handler::RenderErrorPolicy::StopRendering
    }
}

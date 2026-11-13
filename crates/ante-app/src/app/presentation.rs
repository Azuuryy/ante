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
        assets::{catalog::Atlases, sounds::Sfx},
        diagnostics::{self, FrameMetrics, FramePacing},
        profile::Profile,
    },
};
#[derive(Component)]
pub(crate) struct Screen;

#[derive(Component)]
pub(crate) struct PlaybackLabel;

#[derive(Component)]
pub(crate) struct ScoredCard {
    pub(crate) id: CardKey,
}

/// Held Joker, so its scoring step can juice it and show its popup below it.
#[derive(Component)]
pub(crate) struct ScoredJoker {
    pub(crate) id: JokerKey,
}

/// Colored popup over a card while its scoring step shows, like the original status text.
#[derive(Component)]
pub(crate) struct ScorePopup;

type AnimatedCard = (
    Entity,
    &'static ScoredCard,
    &'static mut BorderColor,
    Option<&'static mut HoverMotion>,
    Has<PlayedCard>,
    Option<&'static mut Travel>,
);

type AnimatedJoker = (
    Entity,
    &'static ScoredJoker,
    Option<&'static mut HoverMotion>,
);

#[allow(clippy::too_many_arguments)]
pub(crate) fn animate(
    mut commands: Commands,
    mut session: ResMut<Session>,
    time: Res<Time<Real>>,
    mut text: Query<&mut Text, (With<PlaybackLabel>, Without<run_ui::ScoreTile>)>,
    mut tiles: Query<(&run_ui::ScoreTile, &mut Text, &mut UiTransform), Without<PlaybackLabel>>,
    mut cards: Query<AnimatedCard>,
    (popups, mut jokers): (
        Query<Entity, With<ScorePopup>>,
        Query<AnimatedJoker, Without<ScoredCard>>,
    ),
    (mut metrics, mut shown_popup): (ResMut<FrameMetrics>, Local<Option<(usize, usize)>>),
    (mut sounds, mut room, mut shown_stage): (
        MessageWriter<Sfx>,
        ResMut<RoomShake>,
        Local<Option<PlaybackStage>>,
    ),
) {
    if session.navigation.screen != Page::Run {
        return;
    }
    let speed = session.speed;
    let Some((_, playback)) = &mut session.bypass_change_detection().playback else {
        *shown_popup = None;
        return;
    };
    let start = diagnostics::measurement();
    if playback.advance(time.delta(), speed) {
        session.playback = None;
        *shown_popup = None;
    } else {
        let lifted = matches!(playback.stage(), PlaybackStage::Scoring);
        let discarding = matches!(playback.stage(), PlaybackStage::Discard);
        let scoring = playback.scoring_cards();
        // Each popup of a step shows in turn and juices its source, like `card_eval_status_text`.
        let current = playback.current_popup();
        let key = current.map(|(index, position, _)| (index, position));
        let new_popup = key != *shown_popup;
        if new_popup {
            *shown_popup = key;
            for popup in &popups {
                commands.entity(popup).despawn();
            }
        }
        if let Some((_, _, popup)) = current.filter(|_| new_popup) {
            sounds.write(popup_sound(popup));
            // `card_eval_status_text` shakes the room by 0.7 per popup.
            room.kick(0.7);
        }
        let stage = playback.stage();
        if *shown_stage != Some(stage) {
            *shown_stage = Some(stage);
            if stage == PlaybackStage::Total {
                sounds.write(Sfx::new("chips2", 1.0, 1.0));
            }
        }
        let popup = current.filter(|_| new_popup).and_then(|(index, _, popup)| {
            playback
                .current_step()
                .filter(|(step, _)| *step == index)
                .map(|(_, step)| (step.source, popup))
        });
        for (entity, card, _, motion, played, travel) in &mut cards {
            if discarding
                && played
                && let Some(mut travel) = travel
            {
                // Played cards go to the discard pile one by one, 0.1 game seconds apart.
                let delay = travel.order() as f32 * DRAW_DELAY / speed.factor();
                travel.discard(delay);
            }
            let Some(mut motion) = motion else { continue };
            if played {
                let raise = if lifted && scoring.contains(&card.id) {
                    HIGHLIGHT_LIFT
                } else {
                    0.0
                };
                if motion.raise != raise {
                    motion.raise = raise;
                }
            }
            if let Some((ScoreSource::Card { id, .. }, popup)) = &popup
                && *id == card.id
            {
                motion.juice_up(0.6, 0.1, rand::random());
                let (label, color) = popup_text(*popup);
                commands
                    .entity(entity)
                    .with_child(score_popup(label, color, PopupSide::Above));
            }
        }
        if let Some((ScoreSource::Joker { id, .. }, popup)) = &popup {
            for (entity, joker, motion) in &mut jokers {
                if joker.id != *id {
                    continue;
                }
                if let Some(mut motion) = motion {
                    motion.juice_up(0.6, 0.1, rand::random());
                }
                let (label, color) = popup_text(*popup);
                commands
                    .entity(entity)
                    .with_child(score_popup(label, color, PopupSide::Below));
            }
        }
        for mut text in &mut text {
            let next = playback.description();
            if next != text.0 {
                text.0 = next;
            }
        }
        if let Some((chips, mult)) = playback.score_values() {
            let pop = 1.0 + 0.12 * (1.0 - playback.step_progress());
            for (tile, mut text, mut transform) in &mut tiles {
                let next = if tile.0 {
                    run_ui::score_text(chips)
                } else {
                    run_ui::mult_text(mult)
                };
                if text.0 != next {
                    text.0 = next;
                }
                if transform.scale != Vec2::splat(pop) {
                    transform.scale = Vec2::splat(pop);
                }
            }
        }
        let focused = playback.focused_card();
        for (_, card, mut background, ..) in &mut cards {
            let next = BorderColor::all(if focused == Some(card.id) {
                INK
            } else {
                Color::NONE
            });
            if *background != next {
                *background = next;
            }
        }
    }
    metrics.record_playback(start.elapsed());
}

/// Sound of a popup, as in `card_eval_status_text`: pitch 0.8 + 0.2 × (0.9 to 1.1).
fn popup_sound(popup: Popup) -> Sfx {
    let pitch = 0.8 + 0.2 * (0.9 + 0.2 * rand::random::<f32>());
    match popup {
        Popup::Again => Sfx::new("generic1", pitch, 1.0),
        Popup::Chips(_) => Sfx::new("chips1", pitch, 1.0),
        Popup::Mult(_) => Sfx::new("multhit1", pitch, 1.0),
        Popup::XMult(_) => Sfx::new("multhit2", pitch, 0.7),
        Popup::Money(_) => Sfx::new("coin3", pitch, 1.0),
    }
}

/// Popup label and color, as in `card_eval_status_text`: chips blue, mult and xmult red,
/// money yellow (red when lost), "Again!" in the filter orange.
fn popup_text(popup: Popup) -> (String, Color) {
    match popup {
        Popup::Again => ("Again!".into(), POPUP_AGAIN),
        Popup::Chips(amount) => (format!("{amount:+}"), POPUP_CHIPS),
        Popup::Mult(amount) => (format!("{amount:+} Mult"), POPUP_MULT),
        Popup::XMult(hundredths) => (format!("X{} Mult", multiplier(hundredths)), POPUP_MULT),
        Popup::Money(amount) if amount < 0 => (format!("-${}", -amount), POPUP_MULT),
        Popup::Money(amount) => (format!("${amount}"), POPUP_MONEY),
    }
}

/// A factor in hundredths written like the original: X2, X1.5, X1.25.
fn multiplier(hundredths: usize) -> String {
    let text = format!("{}.{:02}", hundredths / 100, hundredths % 100);
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

const POPUP_AGAIN: Color = Color::srgb(1.0, 0.604, 0.0);
const POPUP_MONEY: Color = Color::srgb(0.953, 0.725, 0.357);
const POPUP_CHIPS: Color = Color::srgb(0.0, 0.616, 1.0);
const POPUP_MULT: Color = Color::srgb(0.996, 0.373, 0.333);

/// Original status text sits above played cards and below Jokers and consumables.
#[derive(Clone, Copy)]
enum PopupSide {
    Above,
    Below,
}

fn score_popup(label: String, color: Color, side: PopupSide) -> impl Bundle {
    let (top, bottom) = match side {
        PopupSide::Above => (px(-34), Val::Auto),
        PopupSide::Below => (Val::Auto, px(-34)),
    };
    (
        ScorePopup,
        Node {
            position_type: PositionType::Absolute,
            top,
            bottom,
            padding: UiRect::axes(px(8), px(2)),
            border_radius: BorderRadius::all(px(6)),
            ..default()
        },
        BackgroundColor(color),
        GlobalZIndex(500),
        Pickable::IGNORE,
        children![(
            Text::new(label),
            TextFont {
                font_size: bevy::text::FontSize::Px(22.0),
                ..default()
            },
            TextColor(Color::WHITE),
            Pickable::IGNORE,
        )],
    )
}

pub(crate) fn render(
    mut commands: Commands,
    session: Res<Session>,
    screens: Query<Entity, With<Screen>>,
    atlases: Res<Atlases>,
    (profile, backend, ui_text): (
        Res<Profile>,
        Res<Backend>,
        Option<Res<crate::services::assets::ui_text::UiText>>,
    ),
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
        let deck = match session.last_config.map(|config| config.deck) {
            Some(ante_core::contract::Deck::Blue) => DeckChoice::Blue,
            _ => DeckChoice::Red,
        };
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
                            run_ui::deck_select(modal, &session, &atlases, &*backend.0)
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
            main_menu(ui, &atlases);
            if page != Page::Main {
                run_ui::overlay(ui, |modal| match page {
                    Page::DeckSelect => run_ui::deck_select(modal, &session, &atlases, &*backend.0),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_sizes_round_to_whole_physical_pixels_and_keep_their_size() {
        for scale in [1.0, 1.12, 1.17, 1.5] {
            let size = snap_font_size(23.36, scale);
            assert!(((size * scale) - (size * scale).round()).abs() < 1e-3);
            assert!((size - 23.36).abs() <= 0.5 / scale + 1e-3);
        }
    }

    #[test]
    fn popups_read_like_the_original_status_text() {
        assert_eq!(multiplier(150), "1.5");
        assert_eq!(multiplier(200), "2");
        assert_eq!(multiplier(125), "1.25");
        assert_eq!(popup_text(Popup::XMult(150)).0, "X1.5 Mult");
        assert_eq!(popup_text(Popup::Mult(4)).0, "+4 Mult");
        assert_eq!(popup_text(Popup::Chips(11)).0, "+11");
        assert_eq!(popup_text(Popup::Money(3)).0, "$3");
        assert_eq!(popup_text(Popup::Money(-2)).0, "-$2");
    }
}

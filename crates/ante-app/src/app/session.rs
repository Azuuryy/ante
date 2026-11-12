use ante_core::contract::{
    CampaignGoal, Command as MvpAction, ContentGroup, GameBackend, Phase, RunConfig, RunDriver,
    RunError as ActionError, RunView, TraceMode,
};
use bevy::{
    clipboard::Clipboard,
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    prelude::*,
    window::PrimaryWindow,
};
#[cfg(feature = "dev-tools")]
use serde_json::json;

use crate::{
    app::{
        navigation::{DeckChoice, Navigation, STAKES, Screen as Page},
        playback::{Playback, Speed},
    },
    components::tooltip,
    journal_event,
    services::{
        assets::import::{self as local_assets, AssetSettings},
        diagnostics::{self, FrameMetrics},
        profile::Profile,
    },
};

#[derive(Resource)]
pub(crate) struct Backend(pub(crate) Box<dyn GameBackend>);

/// The Dev build starts each run with $1000 to test shops quickly.
const STARTING_MONEY_OVERRIDE: Option<usize> = if cfg!(feature = "dev-tools") {
    Some(1000)
} else {
    None
};

#[derive(Resource)]
pub(crate) struct Session {
    pub(crate) seed: String,
    pub(crate) editing_seed: bool,
    pub(crate) seeded_run: bool,
    pub(crate) stake: usize,
    pub(crate) run: Option<Box<dyn RunDriver>>,
    pub(crate) confirmed: Option<RunView>,
    pub(crate) run_id: u64,
    pub(crate) goal: CampaignGoal,
    pub(crate) deck_remaining: bool,
    pub(crate) deck_hovered: bool,
    pub(crate) inventory_selection: Option<(crate::components::card_drag::Lane, usize, u64)>,
    pub(crate) shop_selection: Option<MvpAction>,
    pub(crate) error: Option<String>,
    pub(crate) faulted: bool,
    pub(crate) navigation: Navigation,
    pub(crate) last_config: Option<RunConfig>,
    #[cfg(feature = "dev-tools")]
    pub(crate) action_id: u64,
    #[cfg(feature = "dev-tools")]
    pub(crate) debug_panel: bool,
    pub(crate) speed: Speed,
    pub(crate) playback: Option<(RunView, Playback)>,
    pub(crate) options_tab: u8,
    pub(crate) asset_settings: AssetSettings,
    pub(crate) run_info_tab: u8,
    pub(crate) collection_view: CollectionView,
}

impl Session {
    pub(crate) fn new(goal: CampaignGoal, asset_settings: AssetSettings) -> Self {
        Self {
            seed: String::new(),
            editing_seed: false,
            seeded_run: false,
            stake: 0,
            run: None,
            confirmed: None,
            run_id: 0,
            goal,
            deck_remaining: false,
            deck_hovered: false,
            inventory_selection: None,
            shop_selection: None,
            error: None,
            faulted: false,
            navigation: Navigation::default(),
            last_config: None,
            #[cfg(feature = "dev-tools")]
            action_id: 0,
            #[cfg(feature = "dev-tools")]
            debug_panel: false,
            speed: Speed::default(),
            playback: None,
            options_tab: 0,
            asset_settings,
            run_info_tab: 0,
            collection_view: CollectionView::Hub,
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum CollectionView {
    #[default]
    Hub,
    Decks,
    /// One page of a content group.
    Items(ContentGroup),
}

#[derive(Clone, Copy, Message)]
pub(crate) enum Click {
    EditSeed,
    ToggleSeeded,
    PasteSeed,
    ShiftStake(isize),
    OpenPause,
    OpenRunInfo,
    RunInfoTab(u8),
    OpenDeck,
    HoverDeck(bool),
    SelectOffer {
        action: MvpAction,
        revision: u64,
    },
    ShowRemaining(bool),
    MainMenu,
    ContinueRun,
    Start,
    Restart,
    Menu,
    OpenDecks,
    OpenOptions,
    Back,
    ShiftDeck(isize),
    ToggleVsync,
    Quit,
    Play(MvpAction),
    SelectInventory {
        lane: crate::components::card_drag::Lane,
        index: usize,
        revision: u64,
    },
    ClearSelection,
    Collection(usize),
    CollectionJokers,
    CollectionItems(CollectionView),
    CollectionDecks,
    OptionsTab(u8),
    AssetEdit,
    AssetPaste,
    AssetSteam,
    AssetImport,
    AssetRemove,
    AssetCancelRemove,
    ChangeSpeed,
    /// Steps the master volume, or the game sounds volume when true.
    StepVolume(bool),
    Skip,
    #[cfg_attr(not(feature = "dev-tools"), allow(dead_code))]
    UnlockAll,
}

pub(crate) fn process_clicks(
    mut clicks: MessageReader<Click>,
    mut session: ResMut<Session>,
    mut metrics: ResMut<FrameMetrics>,
    mut profile: ResMut<Profile>,
    mut pacing_toggle: MessageWriter<diagnostics::ToggleFramePacing>,
    (mut exit, mut clipboard, backend): (MessageWriter<AppExit>, ResMut<Clipboard>, Res<Backend>),
    (mut tooltip, drag): (
        Option<ResMut<tooltip::TooltipState>>,
        Option<Res<crate::components::card_drag::CardDrag>>,
    ),
) {
    for &click in clicks.read() {
        if matches!(click, Click::HoverDeck(_)) && drag.as_ref().is_some_and(|drag| drag.active()) {
            continue;
        }
        if matches!(click, Click::Play(_))
            && (session.playback.is_some() || session.navigation.screen != Page::Run)
        {
            continue;
        }
        if session.faulted && !matches!(click, Click::Menu) {
            continue;
        }
        if !matches!(click, Click::HoverDeck(_)) {
            session.error = None;
            if let Some(tooltip) = tooltip.as_mut() {
                tooltip.dismiss();
            }
        }

        const UNLOCK_ALL_GROUPS: [ContentGroup; 11] = ContentGroup::ALL;

        match click {
            Click::SelectInventory {
                lane,
                index,
                revision,
            } => {
                if session.navigation.screen == Page::Run
                    && session.playback.is_none()
                    && session
                        .confirmed
                        .as_ref()
                        .is_some_and(|view| view.revision == revision)
                {
                    let selection = (lane, index, revision);
                    session.inventory_selection =
                        (session.inventory_selection != Some(selection)).then_some(selection);
                }
            }
            Click::ClearSelection => {
                let selected = session
                    .confirmed
                    .as_ref()
                    .into_iter()
                    .flat_map(|view| view.cards.iter())
                    .filter(|card| card.selected)
                    .map(|card| card.id)
                    .collect::<Vec<_>>();
                for id in selected {
                    apply_card_selection(&mut session, &mut metrics, id);
                    if session.error.is_some() {
                        break;
                    }
                }
            }
            Click::EditSeed => session.editing_seed = session.seeded_run,
            Click::ToggleSeeded => {
                session.seeded_run = !session.seeded_run;
                session.editing_seed = session.seeded_run;
            }
            Click::PasteSeed => {
                if session.seeded_run {
                    match clipboard.fetch_text().poll_result() {
                        Some(Ok(text)) if parse_seed(&*backend.0, &text).is_some() => {
                            session.seed = text.trim().to_ascii_uppercase();
                            session.editing_seed = true;
                        }
                        Some(Ok(_)) => session.error = Some("Clipboard has no valid seed.".into()),
                        _ => session.error = Some("Clipboard text is unavailable.".into()),
                    }
                }
            }
            Click::ShiftStake(offset) => {
                session.stake = session
                    .stake
                    .saturating_add_signed(offset)
                    .min(STAKES.len() - 1);
            }
            Click::OpenPause => {
                session.deck_hovered = false;
                if session.confirmed.as_ref().is_some_and(|run| {
                    session.playback.is_some() || !matches!(run.phase, Phase::Won | Phase::Lost)
                }) {
                    session.navigation.screen = Page::Pause;
                }
            }
            Click::OpenRunInfo => {
                session.deck_hovered = false;
                session.run_info_tab = 0;
                session.navigation.open_run_info();
            }
            Click::RunInfoTab(tab) => session.run_info_tab = tab,
            Click::HoverDeck(hover) => {
                session.deck_hovered = hover && session.navigation.screen == Page::Run
            }
            Click::SelectOffer { action, revision } => {
                if session.navigation.screen == Page::Run
                    && session
                        .confirmed
                        .as_ref()
                        .is_some_and(|view| view.phase == Phase::Shop && view.revision == revision)
                {
                    session.shop_selection = Some(action);
                }
            }
            Click::OpenDeck => {
                session.deck_hovered = false;
                if session.navigation.screen == Page::Run
                    && let Some(run) = &session.confirmed
                {
                    let phase = session
                        .playback
                        .as_ref()
                        .map_or_else(|| run.phase, |(view, _)| view.phase);
                    if !matches!(phase, Phase::Won | Phase::Lost) {
                        session.deck_remaining = phase == Phase::Playing;
                        session.navigation.screen = Page::DeckInspect;
                    }
                }
            }
            Click::ShowRemaining(remaining) => {
                if session.navigation.screen == Page::DeckInspect {
                    session.deck_remaining = remaining;
                }
            }
            Click::MainMenu => {
                if session
                    .confirmed
                    .as_ref()
                    .is_some_and(|run| matches!(run.phase, Phase::Won | Phase::Lost))
                {
                    session.run = None;
                    session.confirmed = None;
                    session.playback = None;
                }
                session.navigation.screen = Page::Main;
            }
            Click::ContinueRun => {
                if session.run.is_some() {
                    session.navigation.screen = Page::Run;
                }
            }
            Click::Start | Click::Restart => {
                let config = if matches!(click, Click::Restart) {
                    session.last_config
                } else {
                    session
                        .navigation
                        .deck
                        .and_then(DeckChoice::deck)
                        .filter(|_| session.stake == 0)
                        .and_then(|deck| {
                            let seed = if session.seeded_run {
                                parse_seed(&*backend.0, &session.seed)?
                            } else {
                                rand::random()
                            };
                            Some(RunConfig {
                                seed,
                                deck,
                                goal: session.goal,
                                starting_money: STARTING_MONEY_OVERRIDE,
                            })
                        })
                };
                if let Some(config) = config {
                    let (run, view) = match guard_simulation("Could not start the run", || {
                        let run = backend.0.start(config);
                        let view = run.view();
                        Ok((run, view))
                    }) {
                        Ok(started) => started,
                        Err(error) => {
                            session.faulted = true;
                            session.error = Some(error.to_string());
                            continue;
                        }
                    };
                    session.playback = None;
                    session.editing_seed = false;
                    session.inventory_selection = None;
                    session.shop_selection = None;
                    session.deck_hovered = false;
                    session.confirmed = Some(view);
                    session.run = Some(run);
                    session.run_id = session.run_id.wrapping_add(1);
                    session.last_config = Some(config);
                    session.navigation.screen = Page::Run;
                    journal_event!(
                        "run_started",
                        json!({"seed": config.seed, "deck": format!("{:?}", config.deck)}),
                    );
                } else {
                    session.error = Some(
                        "Choose an available deck, White Stake and a valid seed if enabled.".into(),
                    );
                }
            }
            Click::Menu => {
                session.run = None;
                session.confirmed = None;
                session.playback = None;
                session.editing_seed = false;
                session.faulted = false;
                session.navigation.screen = Page::Main;
                session.navigation.open_decks();
            }
            Click::OpenDecks => {
                session.navigation.open_decks();
                session.editing_seed = false;
            }
            Click::OpenOptions => {
                session.deck_hovered = false;
                session.options_tab = 0;
                session.navigation.open_options();
                session.editing_seed = false;
            }
            Click::Back => {
                session.asset_settings.editing = false;
                session.asset_settings.confirm_remove = false;
                if session.navigation.screen == Page::Collection
                    && session.collection_view != CollectionView::Hub
                {
                    session.collection_view = CollectionView::Hub;
                } else {
                    session.navigation.back();
                }
                session.editing_seed = false;
            }
            Click::ShiftDeck(offset) => {
                if let Some(deck) = session.navigation.deck {
                    session.navigation.deck = Some(deck.shifted(offset));
                }
            }
            Click::ToggleVsync => {
                pacing_toggle.write(diagnostics::ToggleFramePacing);
            }
            Click::Quit => {
                exit.write(AppExit::Success);
            }
            Click::Collection(page) => {
                if session.navigation.screen != Page::Collection {
                    session.collection_view = CollectionView::Hub;
                }
                session.navigation.open_collection(page);
                session.editing_seed = false;
            }
            Click::CollectionJokers => {
                session.collection_view = CollectionView::Items(ContentGroup::Jokers);
                session.navigation.collection_page = 0;
            }
            Click::CollectionItems(category) => {
                session.collection_view = category;
                session.navigation.collection_page = 0;
            }
            Click::CollectionDecks => {
                session.collection_view = CollectionView::Decks;
                session.navigation.deck.get_or_insert(DeckChoice::Red);
            }
            Click::AssetEdit => session.asset_settings.editing = true,
            Click::AssetPaste => {
                if let Some(Ok(text)) = clipboard.fetch_text().poll_result() {
                    session.asset_settings.source = text.trim().to_owned();
                } else {
                    session.asset_settings.status = "Clipboard text unavailable".into();
                }
            }
            Click::AssetSteam => {
                if let Some(path) = local_assets::steam_source() {
                    session.asset_settings.source = path.to_string_lossy().into();
                } else {
                    session.asset_settings.status = "Steam default path unavailable".into();
                }
            }
            Click::AssetImport => {
                session.asset_settings.editing = false;
                session.asset_settings.confirm_remove = false;
                session.asset_settings.status = match session.asset_settings.import() {
                    Ok(count) => format!("{count} files installed. Restart Ante to apply."),
                    Err(error) => format!("Import failed: {error}"),
                };
            }
            Click::AssetRemove => {
                if session.asset_settings.confirm_remove {
                    session.asset_settings.status = match session.asset_settings.remove() {
                        Ok(()) => "Local copy removed. Restart Ante to apply.".into(),
                        Err(error) => format!("Could not remove all files: {error}"),
                    };
                    session.asset_settings.confirm_remove = false;
                } else {
                    session.asset_settings.confirm_remove = true;
                }
            }
            Click::AssetCancelRemove => session.asset_settings.confirm_remove = false,
            Click::OptionsTab(tab) => {
                session.asset_settings.editing = false;
                session.asset_settings.confirm_remove = false;
                if tab < 4 {
                    session.options_tab = tab;
                }
            }
            Click::StepVolume(game) => {
                if let Err(_error) = profile.step_volume(game) {
                    journal_event!("profile_write_failed", json!({"error": _error.to_string()}));
                }
            }
            Click::ChangeSpeed => {
                session.speed = session.speed.next();
                if let Err(_error) = profile.set_game_speed(session.speed.factor()) {
                    journal_event!("profile_write_failed", json!({"error": _error.to_string()}));
                }
            }
            Click::UnlockAll => {
                let ids: Vec<&'static str> = UNLOCK_ALL_GROUPS
                    .into_iter()
                    .flat_map(|group| backend.0.catalog(group).iter().map(|item| item.id.0))
                    .collect();
                #[cfg_attr(not(feature = "dev-tools"), allow(unused_variables))]
                let entries = ids.len();
                match profile.unlock_all(ids) {
                    Ok(()) => journal_event!("profile_unlock_all", json!({"entries": entries})),
                    Err(error) => {
                        journal_event!("profile_write_failed", json!({"error": error.to_string()}));
                        session.error = Some(format!("Collection could not be unlocked: {error}"));
                    }
                }
            }
            Click::Skip => session.playback = None,
            Click::Play(action) => {
                session.inventory_selection = None;
                session.shop_selection = None;
                session.deck_hovered = false;
                #[cfg(feature = "dev-tools")]
                let action_id = {
                    session.action_id += 1;
                    session.action_id
                };
                let Some(before) = session.confirmed.take() else {
                    continue;
                };
                let Some(run) = session.run.as_mut() else {
                    session.faulted = true;
                    session.error = Some("The run is unavailable. Return to the seed menu.".into());
                    continue;
                };
                journal_event!(
                    "action_started",
                    json!({
                        "id": action_id, "seed": before.seed, "phase": format!("{:?}", before.phase),
                        "action": format!("{action:?}"),
                    }),
                );
                let start = diagnostics::measurement();
                let result = guard_simulation("Could not apply the game action", || {
                    run.apply(
                        action,
                        if cfg!(feature = "dev-tools") {
                            TraceMode::Profile
                        } else {
                            TraceMode::Capture
                        },
                    )
                });
                let elapsed = start.elapsed();
                match result {
                    Ok(update) => {
                        session.confirmed = Some(update.view);
                        let outcome = update.outcome;
                        metrics.record_simulation(elapsed, outcome.trace_time);
                        #[cfg(feature = "dev-tools")]
                        if outcome.campaign_won && !before.campaign_won {
                            journal_event!(
                                "run_victory",
                                json!({"seed": before.seed, "ante": 8, "score": outcome.round_score}),
                            );
                        }
                        journal_event!(
                            "action_accepted",
                            json!({
                                "id": action_id, "seed": before.seed, "action": format!("{action:?}"),
                                "before": format!("{:?}", before.phase), "after": format!("{:?}", outcome.phase),
                                "score": outcome.round_score, "last_hand": outcome.hand_score,
                                "trace_steps": outcome.trace.as_ref().map_or(0, |trace| trace.0.len()),
                                "elapsed_us": elapsed.as_micros(), "trace_us": outcome.trace_time.as_micros(),
                            }),
                        );
                        if let Some(id) = update.events.iter().find_map(|event| match event {
                            ante_core::contract::RunEvent::Discovered(id) => Some(id.0),
                            _ => None,
                        }) && let Err(error) = profile.discover(id)
                        {
                            journal_event!(
                                "profile_write_failed",
                                json!({"error": error.to_string()}),
                            );
                            session.error = Some(format!("Collection could not be saved: {error}"));
                        }
                        if let Some(trace) = outcome.trace {
                            let played = before.cards.iter().filter(|card| card.selected).count();
                            session.playback = Some((before, Playback::new(trace, played)));
                        }
                    }
                    Err(error) => {
                        metrics.record_simulation(elapsed, default());
                        session.faulted = matches!(error, ActionError::Simulation(_));
                        journal_event!(
                            if session.faulted {
                                "simulation_error"
                            } else {
                                "action_refused"
                            },
                            json!({
                                "id": action_id, "seed": before.seed, "phase": format!("{:?}", before.phase),
                                "action": format!("{action:?}"), "error": error.to_string(),
                            }),
                        );
                        session.confirmed = Some(before);
                        session.error = Some(error.to_string());
                    }
                }
            }
        }
    }
}

fn guard_simulation<T>(
    context: &str,
    operation: impl FnOnce() -> Result<T, ActionError>,
) -> Result<T, ActionError> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation)).unwrap_or_else(|panic| {
        let detail = panic
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| panic.downcast_ref::<&str>().copied())
            .unwrap_or("unexpected simulation failure");
        Err(ActionError::Simulation(format!("{context}. {detail}")))
    })
}

fn apply_card_selection(
    session: &mut Session,
    metrics: &mut FrameMetrics,
    id: ante_core::contract::CardKey,
) {
    let Some(before) = session.confirmed.take() else {
        return;
    };
    let Some(run) = session.run.as_mut() else {
        session.confirmed = Some(before);
        return;
    };
    let start = diagnostics::measurement();
    let result = guard_simulation("Could not change the card selection", || {
        run.apply(MvpAction::ToggleCard(id), TraceMode::Off)
    });
    metrics.record_simulation(start.elapsed(), default());
    match result {
        Ok(update) => session.confirmed = Some(update.view),
        Err(error) => {
            session.faulted = matches!(error, ActionError::Simulation(_));
            session.confirmed = Some(before);
            session.error = Some(error.to_string());
        }
    }
}

pub(crate) fn right_click_clear(
    mouse: Res<ButtonInput<MouseButton>>,
    session: Res<Session>,
    mut clicks: MessageWriter<Click>,
) {
    if mouse.just_pressed(MouseButton::Right)
        && session.navigation.screen == Page::Run
        && session.playback.is_none()
        && session
            .confirmed
            .as_ref()
            .is_some_and(|view| view.phase == Phase::Playing)
    {
        clicks.write(Click::ClearSelection);
    }
}

pub(crate) fn parse_seed(backend: &dyn GameBackend, text: &str) -> Option<u64> {
    let text = text.trim();
    if text.is_empty() || text.len() > 20 {
        return None;
    }
    if text.bytes().all(|byte| byte.is_ascii_digit()) {
        return text.parse().ok();
    }
    (text.len() <= 8 && text.bytes().all(|byte| byte.is_ascii_alphanumeric()))
        .then(|| backend.parse_seed(&text.to_ascii_uppercase()))
}

pub(crate) fn edit_seed(
    mut keys: MessageReader<KeyboardInput>,
    held: Res<ButtonInput<KeyCode>>,
    primary: Query<Entity, With<PrimaryWindow>>,
    mut session: ResMut<Session>,
    mut clicks: MessageWriter<Click>,
) {
    let Ok(window) = primary.single() else {
        return;
    };
    for key in keys.read() {
        if key.state != ButtonState::Pressed || key.window != window {
            continue;
        }
        if session.navigation.screen == Page::Options
            && session.options_tab == 2
            && session.asset_settings.editing
        {
            if key.key_code == KeyCode::Escape || key.key_code == KeyCode::Enter {
                session.asset_settings.editing = false;
            } else if held.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
                if key.key_code == KeyCode::KeyV && !key.repeat {
                    clicks.write(Click::AssetPaste);
                }
            } else {
                match &key.logical_key {
                    Key::Character(text)
                        if session.asset_settings.source.len() + text.len() <= 4096 =>
                    {
                        session.asset_settings.source.push_str(text)
                    }
                    Key::Backspace => {
                        session.asset_settings.source.pop();
                    }
                    Key::Space => session.asset_settings.source.push(' '),
                    _ => {}
                }
            }
            continue;
        }
        if key.key_code == KeyCode::Escape && !key.repeat {
            if session.editing_seed {
                session.editing_seed = false;
            } else if session.navigation.screen == Page::Run {
                clicks.write(Click::OpenPause);
            } else if session.navigation.screen != Page::Main {
                clicks.write(Click::Back);
            }
            continue;
        }
        if !session.editing_seed || session.navigation.screen != Page::DeckSelect {
            continue;
        }
        if held.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
            if key.key_code == KeyCode::KeyV && !key.repeat {
                clicks.write(Click::PasteSeed);
            }
            continue;
        }
        match &key.logical_key {
            Key::Character(text) if text.bytes().all(|byte| byte.is_ascii_alphanumeric()) => {
                let numeric = session
                    .seed
                    .bytes()
                    .chain(text.bytes())
                    .all(|byte| byte.is_ascii_digit());
                let limit = if numeric { 20 } else { 8 };
                if session.seed.len() + text.len() <= limit {
                    session.seed.push_str(&text.to_ascii_uppercase());
                    session.error = None;
                }
            }
            Key::Backspace => {
                session.seed.pop();
                session.error = None;
            }
            Key::Enter => {
                clicks.write(Click::Start);
            }
            _ => {}
        }
    }
}

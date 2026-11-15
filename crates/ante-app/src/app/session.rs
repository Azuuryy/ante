use ante_core::contract::{
    CampaignGoal, Command, ContentGroup, GameBackend, Phase, RunConfig, RunDriver, RunError,
    RunView, TraceMode,
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

mod actions;
mod click;
mod seed;
mod selection;

use actions::*;
pub(crate) use click::*;
pub(crate) use seed::*;
pub(crate) use selection::*;

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
    pub(crate) shop_selection: Option<Command>,
    /// The current run's command log (`services::run_log`).
    pub(crate) run_log: Option<crate::services::run_log::RunLogFile>,
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
            run_log: None,
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
                    && session.confirmed.as_ref().is_some_and(|view| {
                        matches!(view.phase, Phase::Shop | Phase::Pack) && view.revision == revision
                    })
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
            Click::Start | Click::Restart => start_run(&mut session, &*backend.0, click),
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
            Click::Play(action) => play(&mut session, &mut metrics, &mut profile, action),
        }
    }
}

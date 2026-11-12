use crate::{
    app::{
        navigation::{DeckChoice, Screen as Page},
        presentation::{Screen, fit_room, render},
        session::{Backend, Click, CollectionView, Session, parse_seed, process_clicks},
    },
    components::tiles as menu_ui,
    screens::run as run_ui,
    services::{
        assets::{catalog::Atlases, import::AssetSettings},
        diagnostics::{self, FrameMetrics, FramePacing},
        profile::Profile,
    },
};
use ante_core::contract::{CampaignGoal, Command as MvpAction, Phase};
use bevy::{clipboard::Clipboard, prelude::*, window::PrimaryWindow};

fn app() -> App {
    let mut app = App::new();
    app.insert_resource(Backend(Box::new(ante_core::native::NativeBackend)))
        .insert_resource(Session::new(CampaignGoal::Standard, AssetSettings::load()))
        .init_resource::<FrameMetrics>()
        .insert_resource(Profile::load())
        .add_message::<Click>()
        .add_message::<diagnostics::ToggleFramePacing>()
        .add_message::<AppExit>()
        .init_resource::<Clipboard>()
        .add_systems(Update, process_clicks);
    app
}

fn click(app: &mut App, action: Click) {
    app.world_mut().write_message(action);
    app.update();
}

#[test]
fn navigation_and_options_preserve_run_and_retry_configuration() {
    let mut app = app();
    click(&mut app, Click::Collection(0));
    assert_eq!(
        app.world().resource::<Session>().navigation.screen,
        Page::Collection
    );
    click(&mut app, Click::Back);
    assert_eq!(
        app.world().resource::<Session>().navigation.screen,
        Page::Main
    );
    click(&mut app, Click::OpenDecks);
    click(&mut app, Click::ShiftDeck(1));
    click(&mut app, Click::Start);
    assert_eq!(
        app.world().resource::<Session>().last_config.unwrap().deck,
        ante_core::contract::Deck::Blue
    );
    click(&mut app, Click::Play(MvpAction::StartBlind));
    let before = app
        .world()
        .resource::<Session>()
        .run
        .as_ref()
        .unwrap()
        .view();
    assert_eq!((before.hands, before.discards), (5, 3));
    click(&mut app, Click::OpenOptions);
    click(&mut app, Click::Back);
    let session = app.world().resource::<Session>();
    assert_eq!(session.navigation.screen, Page::Run);
    assert_eq!(session.run.as_ref().unwrap().view(), before);
    click(&mut app, Click::Restart);
    let session = app.world().resource::<Session>();
    assert_eq!(
        session.last_config.unwrap().deck,
        ante_core::contract::Deck::Blue
    );
    assert_eq!(
        session.run.as_ref().unwrap().view().phase,
        Phase::BeforeBlind
    );
}

#[test]
fn seeded_run_pause_and_continue_preserve_configuration() {
    let mut app = app();
    click(&mut app, Click::OpenDecks);
    click(&mut app, Click::ToggleSeeded);
    app.world_mut().resource_mut::<Session>().seed = "ante2026".into();
    click(&mut app, Click::ShiftStake(1));
    click(&mut app, Click::Start);
    assert!(app.world().resource::<Session>().run.is_none());
    click(&mut app, Click::ShiftStake(-1));
    click(&mut app, Click::Start);
    let seed = app.world().resource::<Session>().last_config.unwrap().seed;
    assert_eq!(
        seed,
        parse_seed(&ante_core::native::NativeBackend, " ANTE2026 ").unwrap()
    );
    assert_eq!(parse_seed(&ante_core::native::NativeBackend, "0"), Some(0));
    assert_eq!(
        parse_seed(&ante_core::native::NativeBackend, "not a seed"),
        None
    );
    click(&mut app, Click::OpenPause);
    click(&mut app, Click::OpenOptions);
    click(&mut app, Click::Back);
    assert_eq!(
        app.world().resource::<Session>().navigation.screen,
        Page::Pause
    );
    click(&mut app, Click::OpenDecks);
    click(&mut app, Click::Back);
    assert_eq!(
        app.world().resource::<Session>().navigation.screen,
        Page::Pause
    );
    click(&mut app, Click::MainMenu);
    click(&mut app, Click::OpenDecks);
    click(&mut app, Click::ContinueRun);
    assert_eq!(
        app.world().resource::<Session>().navigation.screen,
        Page::Run
    );
    click(&mut app, Click::Restart);
    assert_eq!(
        app.world()
            .resource::<Session>()
            .run
            .as_ref()
            .unwrap()
            .view()
            .seed,
        seed
    );
}

#[test]
fn collection_subpages_return_to_hub_and_options_keep_tab_state() {
    let mut app = app();
    click(&mut app, Click::Collection(0));
    click(&mut app, Click::CollectionJokers);
    assert!(
        app.world().resource::<Session>().collection_view
            == CollectionView::Items(ante_core::contract::ContentGroup::Jokers)
    );
    click(&mut app, Click::Collection(1));
    assert_eq!(
        app.world().resource::<Session>().navigation.collection_page,
        1
    );
    click(&mut app, Click::Back);
    assert!(app.world().resource::<Session>().collection_view == CollectionView::Hub);
    click(&mut app, Click::CollectionDecks);
    click(&mut app, Click::ShiftDeck(1));
    assert_eq!(
        app.world().resource::<Session>().navigation.deck,
        Some(DeckChoice::Blue)
    );
    click(&mut app, Click::Back);
    click(&mut app, Click::Back);
    assert_eq!(
        app.world().resource::<Session>().navigation.screen,
        Page::Main
    );
    click(&mut app, Click::OpenOptions);
    click(&mut app, Click::OptionsTab(1));
    assert_eq!(app.world().resource::<Session>().options_tab, 1);
    click(&mut app, Click::ToggleVsync);
    assert!(
        !app.world()
            .resource::<Messages<diagnostics::ToggleFramePacing>>()
            .is_empty()
    );
    click(&mut app, Click::Back);
    assert_eq!(
        app.world().resource::<Session>().navigation.screen,
        Page::Main
    );
}

#[test]
fn unavailable_decks_cannot_start_and_quit_is_requested() {
    let mut app = app();
    click(&mut app, Click::OpenDecks);
    click(&mut app, Click::ShiftDeck(2));
    click(&mut app, Click::Start);
    assert!(app.world().resource::<Session>().run.is_none());
    assert!(app.world().resource::<Session>().error.is_some());
    click(&mut app, Click::Back);
    click(&mut app, Click::Quit);
    assert!(!app.world().resource::<Messages<AppExit>>().is_empty());
}

#[test]
fn run_board_survives_phase_and_deck_modal_changes() {
    let mut app = app();
    app.init_resource::<Atlases>()
        .insert_resource(FramePacing::new(false))
        .add_systems(Update, render.after(process_clicks));
    click(&mut app, Click::OpenDecks);
    click(&mut app, Click::Start);
    let root = app
        .world_mut()
        .query_filtered::<Entity, With<Screen>>()
        .single(app.world())
        .unwrap();
    let mut tiles: Vec<_> = app
        .world_mut()
        .query_filtered::<Entity, With<run_ui::ScoreTile>>()
        .iter(app.world())
        .collect();
    tiles.sort_unstable();
    assert_eq!(tiles.len(), 2);
    click(&mut app, Click::Play(MvpAction::StartBlind));
    let id = app
        .world()
        .resource::<Session>()
        .run
        .as_ref()
        .unwrap()
        .view()
        .cards[0]
        .id;
    click(&mut app, Click::Play(MvpAction::ToggleCard(id)));
    let before = app
        .world()
        .resource::<Session>()
        .run
        .as_ref()
        .unwrap()
        .view();
    click(&mut app, Click::OpenDeck);
    assert!(app.world().resource::<Session>().deck_remaining);
    click(&mut app, Click::ShowRemaining(false));
    click(&mut app, Click::Back);
    let session = app.world().resource::<Session>();
    assert_eq!(session.navigation.screen, Page::Run);
    assert_eq!(session.run.as_ref().unwrap().view(), before);
    assert!(app.world().get::<Screen>(root).is_some());
    assert!(
        tiles
            .into_iter()
            .all(|tile| app.world().get::<run_ui::ScoreTile>(tile).is_some())
    );
}

#[test]
fn fit_room_scales_the_screen_root_but_not_the_profiler() {
    let mut app = App::new();
    app.add_systems(Update, fit_room);
    let primary = app
        .world_mut()
        .spawn((
            Window {
                resolution: (1080, 700).into(),
                ..default()
            },
            PrimaryWindow,
        ))
        .id();
    let screen = app
        .world_mut()
        .spawn((Screen, Node::default(), UiTransform::IDENTITY))
        .id();
    let profiler = app
        .world_mut()
        .spawn((UiTargetCamera(Entity::PLACEHOLDER), UiTransform::IDENTITY))
        .id();
    app.update();
    let world = app.world();
    let (canvas, scale) = menu_ui::viewport_fit(Vec2::new(1080.0, 700.0));
    assert_eq!(world.get::<Window>(primary).unwrap().scale_factor(), scale);
    assert_eq!(world.get::<Node>(screen).unwrap().width, px(canvas.x));
    assert_eq!(
        *world.get::<UiTransform>(screen).unwrap(),
        UiTransform::IDENTITY
    );
    assert_eq!(
        *world.get::<UiTransform>(profiler).unwrap(),
        UiTransform::IDENTITY
    );
}

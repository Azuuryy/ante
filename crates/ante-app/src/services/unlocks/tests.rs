use super::*;
use ante_core::{
    contract::{CampaignGoal, Deck, RunConfig, RunDriver, TraceMode},
    engine::{EngineRun, ExternalBackend},
};

fn profile(name: &str) -> (Profile, std::path::PathBuf) {
    let path =
        std::env::temp_dir().join(format!("ante-unlocks-{name}-{}.json", std::process::id()));
    let _ = std::fs::remove_file(&path);
    (Profile::fresh_at(path.clone()), path)
}

fn run() -> EngineRun {
    EngineRun::with_config(RunConfig {
        seed: 3,
        deck: Deck::Red,
        goal: CampaignGoal::Standard,
        starting_money: None,
        stake: 1,
    })
}

fn send(
    run: &mut EngineRun,
    tracker: &mut RunTracker,
    profile: &mut Profile,
    command: impl Fn(&RunView) -> Command,
) -> Vec<String> {
    let before = RunDriver::view(run);
    let command = command(&before);
    let update = RunDriver::apply(run, command, TraceMode::Capture).unwrap();
    observe(
        tracker,
        &before,
        command,
        &update,
        profile,
        &ExternalBackend,
    )
}

#[test]
fn a_fresh_profile_locks_what_the_original_locks() {
    let (profile, _) = profile("locked");
    let locked = locked_ids(&profile, &ExternalBackend);
    assert!(locked.contains(&"j_brainstorm".to_owned()));
    assert!(locked.contains(&"v_overstock_plus".to_owned()));
    assert!(!locked.contains(&"j_joker".to_owned()));
    assert!(!locked.contains(&"j_perkeo".to_owned()) || !profile.discovered("j_perkeo"));
}

#[test]
fn played_hands_count_in_the_career() {
    let (mut profile, path) = profile("career");
    let mut tracker = RunTracker::new(false, "b_red", 1);
    let mut game = run();
    send(&mut game, &mut tracker, &mut profile, |_| {
        Command::StartBlind
    });
    for position in 0..3 {
        send(&mut game, &mut tracker, &mut profile, move |view| {
            Command::ToggleCard(view.cards[position].id)
        });
    }
    send(&mut game, &mut tracker, &mut profile, |_| Command::Play);
    assert_eq!(profile.progress().stat("c_hands_played"), 1);
    assert_eq!(profile.progress().stat("c_cards_played"), 3);
    let _ = std::fs::remove_file(path);
}

#[test]
fn career_conditions_unlock_once_met() {
    let (mut profile, path) = profile("acrobat");
    profile
        .update_progress(|progress| progress.add("c_hands_played", 199))
        .unwrap();
    let mut tracker = RunTracker::new(false, "b_red", 1);
    let mut game = run();
    send(&mut game, &mut tracker, &mut profile, |_| {
        Command::StartBlind
    });
    send(&mut game, &mut tracker, &mut profile, |view| {
        Command::ToggleCard(view.cards[0].id)
    });
    let unlocked = send(&mut game, &mut tracker, &mut profile, |_| Command::Play);
    assert!(unlocked.contains(&"Acrobat".to_owned()), "{unlocked:?}");
    assert!(profile.is_unlocked("j_acrobat", Some("Play 200 hands.")));
    let _ = std::fs::remove_file(path);
}

#[test]
fn a_seeded_run_unlocks_nothing() {
    let (mut profile, path) = profile("seeded");
    profile
        .update_progress(|progress| progress.add("c_hands_played", 199))
        .unwrap();
    let mut tracker = RunTracker::new(true, "b_red", 1);
    let mut game = run();
    send(&mut game, &mut tracker, &mut profile, |_| {
        Command::StartBlind
    });
    send(&mut game, &mut tracker, &mut profile, |view| {
        Command::ToggleCard(view.cards[0].id)
    });
    assert!(send(&mut game, &mut tracker, &mut profile, |_| Command::Play).is_empty());
    assert_eq!(
        profile.progress().stat("c_hands_played"),
        200,
        "statistics still count"
    );
    let _ = std::fs::remove_file(path);
}

#[test]
fn deck_wins_unlock_the_next_decks_and_stakes() {
    let (mut profile, path) = profile("decks");
    let view = RunDriver::view(&run());
    let found = Discoveries::default();
    let met = |profile: &Profile, id| {
        conditions::met(id, &Moment::default(), &view, profile.progress(), found)
    };
    assert!(!met(&profile, "b_magic"));
    profile
        .update_progress(|progress| progress.record_win("b_red", 2))
        .unwrap();
    assert!(met(&profile, "b_magic"), "won with the Red Deck");
    assert!(met(&profile, "b_zodiac"), "won at Red Stake");
    assert!(!met(&profile, "b_painted"), "not yet at Green Stake");
    let _ = std::fs::remove_file(path);
}

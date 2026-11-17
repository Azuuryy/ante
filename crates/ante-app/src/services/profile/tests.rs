use super::*;

fn temporary(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("ante-{name}-{}.json", std::process::id()))
}

fn profile_at(path: PathBuf) -> Profile {
    Profile {
        path,
        discovered: BTreeSet::new(),
        all_unlocked: false,
        game_speed: None,
        volumes: Volumes::default(),
        progress: Progress::default(),
        writable: true,
        dirty: false,
    }
}

#[test]
fn volumes_default_to_the_original_and_save_when_asked() {
    let path = temporary("profile-volume");
    let mut profile = profile_at(path.clone());
    let all = [
        VolumeChannel::Master,
        VolumeChannel::Music,
        VolumeChannel::Game,
    ];
    assert_eq!(
        all.map(|channel| profile.volume_of(channel)),
        [50, 100, 100]
    );
    profile.set_volume(VolumeChannel::Master, 64);
    profile.set_volume(VolumeChannel::Music, 130);
    profile.set_volume(VolumeChannel::Game, 0);
    assert!(Profile::read(&path).unwrap().volumes == Volumes::default());
    profile.save_volumes().unwrap();
    let stored = Profile::read(&path).unwrap();
    assert_eq!(
        stored.volumes,
        Volumes {
            master: 64,
            music: 100,
            game: 0
        }
    );
    fs::remove_file(path).unwrap();
}

#[test]
fn version_three_profiles_keep_their_volumes_and_get_full_music() {
    let path = temporary("profile-v3-volume");
    fs::write(
        &path,
        r#"{"version":3,"discovered":[],"volume":30,"game_volume":70}"#,
    )
    .unwrap();
    let stored = Profile::read(&path).unwrap();
    assert_eq!(
        stored.volumes,
        Volumes {
            master: 30,
            music: 100,
            game: 70
        }
    );
    fs::remove_file(path).unwrap();
}

#[test]
fn game_speed_survives_a_save_and_older_versions_have_none() {
    let path = temporary("profile-speed");
    let mut profile = profile_at(path.clone());
    profile.set_game_speed(4.0).unwrap();
    assert_eq!(Profile::read(&path).unwrap().game_speed, Some(4.0));
    fs::write(&path, r#"{"version":2,"discovered":[],"game_speed":8}"#).unwrap();
    assert_eq!(Profile::read(&path).unwrap().game_speed, None);
    fs::remove_file(path).unwrap();
}

#[test]
fn versioned_profile_saves_discoveries_across_writes() {
    let path = temporary("profile");
    let mut profile = profile_at(path.clone());
    profile.discover("j_joker").unwrap();
    profile.discover("j_misprint").unwrap();
    let stored = Profile::read(&path).unwrap();
    assert_eq!(
        stored.discovered,
        BTreeSet::from(["j_joker".into(), "j_misprint".into()])
    );
    assert!(!stored.all_unlocked);
    fs::remove_file(path).unwrap();
}

#[test]
fn version_one_profiles_migrate_without_losing_discoveries() {
    let path = temporary("profile-v1");
    fs::write(&path, r#"{"version":1,"discovered":["j_joker","v_blank"]}"#).unwrap();
    let stored = Profile::read(&path).unwrap();
    assert_eq!(
        stored.discovered,
        BTreeSet::from(["j_joker".into(), "v_blank".into()])
    );
    assert!(!stored.all_unlocked);

    let mut profile = Profile {
        path: path.clone(),
        discovered: stored.discovered,
        all_unlocked: stored.all_unlocked,
        game_speed: stored.game_speed,
        volumes: stored.volumes,
        progress: stored.progress,
        writable: true,
        dirty: false,
    };
    profile.discover("j_brainstorm").unwrap();
    let rewritten: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(rewritten["version"], VERSION);
    assert_eq!(rewritten["all_unlocked"], false);
    fs::remove_file(path).unwrap();
}

#[test]
fn unlock_all_persists_flag_and_catalog_entries() {
    let path = temporary("profile-unlock");
    let mut profile = profile_at(path.clone());
    profile
        .unlock_all(["j_brainstorm", "v_blank", "tag_rare"])
        .unwrap();
    assert!(profile.all_unlocked());
    assert!(profile.discovered("j_brainstorm"));
    assert!(profile.is_unlocked("j_any", Some("Locked")));

    let stored = Profile::read(&path).unwrap();
    assert!(stored.all_unlocked);
    assert_eq!(
        stored.discovered,
        BTreeSet::from(["j_brainstorm".into(), "v_blank".into(), "tag_rare".into()])
    );
    fs::remove_file(path).unwrap();
}

#[test]
fn locked_items_open_with_their_condition_not_with_discovery() {
    let path = temporary("profile-locks");
    let mut profile = profile_at(path.clone());
    assert!(profile.is_unlocked("j_joker", None));
    let condition = Some("Discard a Royal Flush");
    profile.discover("j_brainstorm").unwrap();
    assert!(
        !profile.is_unlocked("j_brainstorm", condition),
        "seen is not unlocked"
    );
    profile
        .update_progress(|progress| {
            progress.unlocked.insert("j_brainstorm".into());
        })
        .unwrap();
    assert!(profile.is_unlocked("j_brainstorm", condition));
    // A Legendary counts once found.
    assert!(!profile.is_unlocked("j_perkeo", Some("Found in the Soul")));
    profile.discover("j_perkeo").unwrap();
    assert!(profile.is_unlocked("j_perkeo", Some("Found in the Soul")));
    fs::remove_file(path).unwrap();
}

#[test]
fn progress_round_trips_through_version_5() {
    let path = temporary("profile-progress");
    let mut profile = profile_at(path.clone());
    profile
        .update_progress(|progress| {
            progress.add("c_hands_played", 3);
            progress.record_win("b_red", 2);
            progress.unlocked.insert("b_blue".into());
        })
        .unwrap();
    let stored = Profile::read(&path).unwrap();
    assert_eq!(stored.progress.stat("c_hands_played"), 3);
    assert_eq!(stored.progress.deck_win_stake("b_red"), 2);
    assert!(stored.progress.unlocked.contains("b_blue"));
    fs::remove_file(path).unwrap();
}

#[test]
fn a_version_4_profile_starts_with_everything_locked() {
    let path = temporary("profile-v4");
    fs::write(
        &path,
        r#"{"version":4,"discovered":["j_brainstorm"],"all_unlocked":false}"#,
    )
    .unwrap();
    let stored = Profile::read(&path).unwrap();
    assert!(stored.progress.unlocked.is_empty());
    fs::remove_file(path).unwrap();
}

#[test]
fn corrupted_profile_is_rejected() {
    let path = temporary("invalid-profile");
    fs::write(&path, "not JSON").unwrap();
    assert!(Profile::read(&path).is_err());
    fs::remove_file(path).unwrap();
}

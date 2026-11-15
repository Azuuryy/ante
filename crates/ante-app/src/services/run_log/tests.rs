use super::*;
use ante_core::{
    contract::{CampaignGoal, Deck, RunDriver, TraceMode},
    engine::EngineRun,
    run_log::replay,
};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ante-run-log-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir
}

const CONFIG: RunConfig = RunConfig {
    seed: 11,
    deck: Deck::Blue,
    goal: CampaignGoal::Standard,
    starting_money: None,
};

#[test]
fn a_written_log_replays_from_its_file() {
    let dir = scratch("replay");
    let mut log = RunLogFile::start_in(&dir, CONFIG);
    let mut run = EngineRun::with_config(CONFIG);
    for command in [Command::StartBlind, Command::SortSuit] {
        let before = RunDriver::view(&run);
        let result = RunDriver::apply(&mut run, command, TraceMode::Off);
        let after = RunDriver::view(&run);
        log.record(&before, command, result.as_ref(), &after);
    }
    let text = fs::read_to_string(log.path()).unwrap();
    assert_eq!(text.lines().count(), 3);
    let report = replay(text.lines()).unwrap();
    assert_eq!((report.steps, report.divergence), (2, None));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn only_the_most_recent_runs_are_kept() {
    let dir = scratch("prune");
    fs::create_dir_all(&dir).unwrap();
    for stamp in 0..KEEP + 3 {
        fs::write(dir.join(format!("{stamp:013}-1.jsonl")), "").unwrap();
    }
    fs::write(dir.join("notes.txt"), "kept").unwrap();
    let log = RunLogFile::start_in(&dir, CONFIG);
    let mut logs: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "jsonl"))
        .collect();
    logs.sort();
    assert_eq!(logs.len(), KEEP);
    assert_eq!(logs.last().unwrap(), log.path());
    assert!(!dir.join(format!("{:013}-1.jsonl", 3)).exists());
    assert!(dir.join(format!("{:013}-1.jsonl", 4)).exists());
    assert!(dir.join("notes.txt").exists());
    fs::remove_dir_all(&dir).unwrap();
}

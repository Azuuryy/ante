use std::{
    backtrace::Backtrace,
    collections::VecDeque,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use bevy::{
    app::AppExit,
    ecs::error::{BevyError, ErrorContext},
    log::BoxedLayer,
    prelude::*,
    render::error_handler::{RenderError, RenderErrorPolicy},
};
use serde_json::{Value, json};
use tracing::{
    Event, Subscriber,
    field::{Field, Visit},
};
use tracing_subscriber::layer::{Context, Layer};

const LOG_BYTES: u64 = 2 * 1024 * 1024;
const RECENT_EVENTS: usize = 128;
static JOURNAL: OnceLock<Journal> = OnceLock::new();

struct Journal {
    dir: PathBuf,
    session: String,
    state: Mutex<State>,
}

#[derive(Default)]
struct State {
    recent: VecDeque<Value>,
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

pub fn init() {
    init_at(crate::services::storage::local_dir().join("logs"));
}

fn init_at(dir: PathBuf) {
    if let Err(error) = fs::create_dir_all(&dir) {
        eprintln!("Ante log directory unavailable: {error}");
    }
    let _ = JOURNAL.set(Journal {
        dir,
        session: format!("{}-{}", unix_ms(), std::process::id()),
        state: Mutex::default(),
    });
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        fatal("panic", &format!("{info}\n{}", Backtrace::force_capture()));
        previous(info);
    }));
    event("session_started", json!({"os": std::env::consts::OS}));
}

fn rotated_append(dir: &Path, line: &[u8]) -> io::Result<()> {
    let path = dir.join("ante.jsonl");
    let current = path.metadata().map_or(0, |m| m.len());
    if current + line.len() as u64 + 1 > LOG_BYTES {
        fs::copy(&path, dir.join("ante.previous.jsonl"))?;
        File::create(&path)?;
    }
    let mut file = OpenOptions::new().append(true).create(true).open(path)?;
    file.write_all(line)?;
    file.write_all(b"\n")
}

fn envelope(journal: &Journal, kind: &str, data: Value) -> Value {
    json!({
        "unix_ms": unix_ms(),
        "session": journal.session,
        "version": env!("CARGO_PKG_VERSION"),
        "event": kind,
        "data": data,
    })
}

pub fn event(kind: &str, data: Value) {
    let Some(journal) = JOURNAL.get() else { return };
    let Ok(mut state) = journal.state.lock() else {
        return;
    };
    let entry = envelope(journal, kind, data);
    if state.recent.len() == RECENT_EVENTS {
        state.recent.pop_front();
    }
    if let Ok(line) = serde_json::to_vec(&entry)
        && line.len() <= 16 * 1024
    {
        if let Err(error) = rotated_append(&journal.dir, &line) {
            eprintln!("Ante could not write to the journal: {error}");
        }
        state.recent.push_back(entry);
    }
}

fn fatal(kind: &str, detail: &str) {
    let Some(journal) = JOURNAL.get() else { return };
    let recent = journal
        .state
        .try_lock()
        .ok()
        .map(|state| state.recent.clone());
    let report = envelope(journal, kind, json!({"detail": detail, "recent": recent}));
    if let Err(error) = fs::write(journal.dir.join("ante-crash.json"), report.to_string()) {
        eprintln!("Ante could not write the crash report: {error}");
    }
}

pub fn export(csv: &str) -> io::Result<PathBuf> {
    let journal = JOURNAL
        .get()
        .ok_or_else(|| io::Error::other("Journal not initialized"))?;
    let state = journal
        .state
        .lock()
        .map_err(|_| io::Error::other("Journal unavailable after a failure"))?;
    let log = |name| match fs::read_to_string(journal.dir.join(name)) {
        Ok(log) => Ok(log),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(error),
    };
    let report = envelope(
        journal,
        "export",
        json!({
            "recent": state.recent,
            "journal": log("ante.jsonl")?,
            "previous_journal": log("ante.previous.jsonl")?,
            "measurements_csv": csv,
        }),
    );
    let path = journal.dir.join("ante-report.json");
    fs::write(&path, report.to_string())?;
    fs::write(journal.dir.join("ante-frames.csv"), csv)?;
    Ok(path)
}

pub fn ecs_error(error: BevyError, context: ErrorContext) {
    event(
        "bevy_error",
        json!({"error": error.to_string(), "context": format!("{context:?}")}),
    );
    panic!("{context:?}: {error}");
}

pub fn render_error(error: &RenderError, world: &mut World, _: &mut World) -> RenderErrorPolicy {
    let message = format!("{:?}: {}", error.ty, error.description);
    event("gpu_error", json!({"error": message}));
    fatal("gpu_error", &message);
    world.write_message(AppExit::error());
    RenderErrorPolicy::StopRendering
}

pub fn bevy_layer(_: &mut App) -> Option<BoxedLayer> {
    Some(Box::new(BevyJournal))
}

struct BevyJournal;

#[derive(Default)]
struct Fields(serde_json::Map<String, Value>);

impl Visit for Fields {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let text = format!("{value:?}");
        self.0.insert(
            field.name().to_owned(),
            text.chars().take(2048).collect::<String>().into(),
        );
    }
}

impl<S: Subscriber> Layer<S> for BevyJournal {
    fn on_event(&self, entry: &Event<'_>, _: Context<'_, S>) {
        let metadata = entry.metadata();
        if *metadata.level() > tracing::Level::WARN {
            return;
        }
        let mut fields = Fields::default();
        entry.record(&mut fields);
        event(
            "bevy_log",
            json!({
                "level": metadata.level().as_str(),
                "target": metadata.target(),
                "fields": fields.0,
            }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "run by the crash-report subprocess test"]
    fn panic_child() {
        let dir = std::env::var_os("ANTE_TEST_CRASH_DIR").unwrap();
        init_at(dir.into());
        event("action_started", json!({"id": 42, "seed": 17}));
        panic!("simulated fatal scoring failure");
    }

    #[test]
    fn panic_report_contains_pending_action() {
        let dir =
            std::env::temp_dir().join(format!("ante-crash-{}-{}", std::process::id(), unix_ms()));
        let (_, module) = module_path!().split_once("::").unwrap();
        let test = format!("{module}::panic_child");
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", &test, "--ignored"])
            .env("ANTE_TEST_CRASH_DIR", &dir)
            .output()
            .unwrap();
        assert!(!child.status.success());
        let report: Value =
            serde_json::from_slice(&fs::read(dir.join("ante-crash.json")).unwrap()).unwrap();
        assert_eq!(report["event"], "panic");
        assert!(
            report["data"]["detail"]
                .as_str()
                .unwrap()
                .contains("simulated fatal scoring failure")
        );
        assert!(
            report["data"]["recent"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| { entry["event"] == "action_started" && entry["data"]["id"] == 42 })
        );
        fs::remove_file(dir.join("ante-crash.json")).unwrap();
        fs::remove_file(dir.join("ante.jsonl")).unwrap();
        fs::remove_dir(dir).unwrap();
    }
}
